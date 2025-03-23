use std::fs::File;
use std::io::Write;
use std::path::Path;
use chrono::Local;
use qrcode::QrCode;

/// Represents information needed to generate a paper wallet
pub struct PaperWalletInfo {
    pub address: String,
    pub mnemonic: String,
    pub word_count: usize,
    pub position: u32,
}

/// Shuffle words in a phrase to create obfuscated version for back of wallet
fn shuffle_phrase(phrase: &str) -> String {
    let mut words: Vec<&str> = phrase.split_whitespace().collect();
    let len = words.len();
    
    // Simple deterministic Fisher-Yates shuffle
    for i in 0..(len - 1) {
        let j = ((i * 13) + 7) % len;
        words.swap(i, j);
    }
    
    words.join(" ")
}

/// Generates a paper wallet HTML file that exactly matches the reference design
pub fn generate_paper_wallet(info: &PaperWalletInfo, output_path: &Path) -> Result<(), String> {
    // Derive a "public key" from the address (using its first 20 characters)
    let pseudo_pub_key = format!("ergo-wallet-key-{}", &info.address[..20]);
    
    // Format mnemonic with proper spacing
    let mnemonic = &info.mnemonic;
    
    // Create shuffled version for the back side of the wallet
    let shuffled = shuffle_phrase(mnemonic);
    
    // Generate the QR code SVG from the address
    let address_qr_svg = create_reliable_qr_svg(&info.address)?;
    
    // Build the HTML content
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Ergo Paper Wallet</title>
    <style>
        /* ------------------------------------------------------
           PRINT STYLES: A4 with no margins
        ------------------------------------------------------ */
        @media print {{
            @page {{
                size: A4;
                margin: 0;
            }}
            body {{
                margin: 0;
            }}
            .no-print {{
                display: none;
            }}
            .sheet-wrap {{
                width: 100%;
                height: 100%;
                margin: 0;
                overflow: hidden;
            }}
            .sheet {{
                width: 210mm;
                height: 297mm;
                margin: 0;
                box-shadow: none;
            }}
        }}
        
        /* ------------------------------------------------------
           GLOBAL BASE STYLES
        ------------------------------------------------------ */
        html, body {{
            padding: 0;
            margin: 0;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            font-size: 14px;
            line-height: 1.5;
            color: #333;
            background-color: #f5f5f5;
        }}
        
        .no-print {{
            position: fixed;
            top: 20px;
            right: 20px;
            padding: 10px 20px;
            background-color: #FF8C00;
            color: white;
            border: none;
            border-radius: 4px;
            font-weight: bold;
            cursor: pointer;
            z-index: 1000;
        }}
        .no-print:hover {{
            background-color: #E67E00;
        }}
        
        /* ------------------------------------------------------
           SCREEN STYLES: FORCE A4 SIZE IN A SCROLLABLE WRAPPER
        ------------------------------------------------------ */
        @media screen {{
            .sheet-wrap {{
                /* This is a fixed-size container: 210mm x 297mm */
                width: 210mm;
                height: 297mm;
                margin: 20px auto;       /* center it */
                box-shadow: 0 0 10px rgba(0, 0, 0, 0.1);
                overflow: auto;          /* scrollbars if window too small */
                background-color: #fff;
            }}
            .sheet {{
                /* Fill the wrapper exactly */
                width: 100%;
                height: 100%;
            }}
        }}
        
        /* ------------------------------------------------------
           2×2 LAYOUT USING GRID (never stacks vertically)
        ------------------------------------------------------ */
        .sheet {{
            display: grid;
            grid-template-columns: 1fr 1fr; /* 2 columns */
            grid-template-rows: 1fr 1fr;    /* 2 rows */
        }}
        
        .quarter {{
            box-sizing: border-box;
            border: 1px dashed #ccc;
            position: relative;
            padding: 0; /* We'll do actual padding inside .row */
            overflow: hidden;
        }}
        
        /* Mark which quadrant is which for dashed border logic */
        .even-top {{
            border-bottom: 1px dashed #ccc;
            border-right: 1px dashed #ccc;
        }}
        .odd-top {{
            border-bottom: 1px dashed #ccc;
        }}
        .even-bottom {{
            border-right: 1px dashed #ccc;
        }}
        
        /* Layout building blocks */
        .col {{
            display: flex;
            flex-direction: column;
            height: 100%;
        }}
        .row {{
            padding: 15px 20px;
        }}
        .flex-grow {{
            flex-grow: 1;
        }}
        
        /* Header area */
        .header {{
            height: 56px;
            background-color: #1A1A1A;
            display: flex;
            align-items: center;
            padding: 0 15px;
            color: white;
        }}
        .logo {{
            height: 36px;
            width: 36px;
            display: flex;
            align-items: center;
            justify-content: center;
            background-color: #FF8C00;
            color: white;
            font-weight: bold;
            font-size: 24px;
            margin-right: 10px;
            clip-path: polygon(30% 0%, 70% 0%, 100% 30%, 100% 70%, 70% 100%, 30% 100%, 0% 70%, 0% 30%);
        }}
        .header-title {{
            font-size: 20px;
            font-weight: 300;
            letter-spacing: 0.05em;
        }}
        
        /* Title */
        .title {{
            text-align: center;
            font-size: 20px;
            margin-bottom: 15px;
            font-weight: 500;
        }}
        
        /* Bordered box */
        .bordered {{
            border: 1px solid #e2e2e2;
            padding: 12px;
            margin: 10px 0;
            border-radius: 3px;
            background-color: #f9f9f9;
        }}
        
        .key {{
            font-family: monospace;
            font-size: 13px;
            line-height: 1.3;
            word-break: break-all;
        }}
        
        .tip-text {{
            border-top: 1px solid #e2e2e2;
            padding-top: 8px;
            margin-top: 10px;
            color: #666;
            font-size: 12px;
        }}
        
        /* QR code area */
        .qr-code {{
            text-align: center;
            margin: 15px 0;
        }}
        
        /* Two classes for different QR sizes */
        .qr-wrapper-large {{
            width: 200px;
            height: 200px;
        }}
        .qr-wrapper-small {{
            width: 120px;
            height: 120px;
        }}
        
        /* Make the SVG scale to its container */
        .qr-wrapper svg {{
            display: block;
            width: 100%;
            height: auto;
            max-width: 100%;
            max-height: 100%;
        }}
        
        /* Flip text for obfuscated seed */
        .transform-rotate {{
            transform: rotate(180deg);
        }}
    </style>
</head>
<body>
    <!-- Print button (hidden in print mode) -->
    <button class="no-print" onclick="window.print()">Print Wallet</button>
    
    <div class="sheet-wrap">
        <div class="sheet">
            <!-- First Quarter (Top Left) - Public Key -->
            <div class="quarter even-top">
                <div class="col">
                    <div class="header">
                        <div class="logo">Σ</div>
                        <span class="header-title">Paper Wallet</span>
                    </div>
                    <div class="row">
                        <h1 class="title">Public Key</h1>
                    </div>
                    <div class="row flex-grow">
                        <div class="qr-code">
                            <!-- Large QR wrapper for the public key quadrant -->
                            <div class="qr-wrapper qr-wrapper-large">
                                {2}
                            </div>
                        </div>
                        <p class="key bordered">
                            {1}
                        </p>
                    </div>
                    <div class="row">
                        <p class="tip-text">
                            Public Key used to generate derived addresses. (not yet functional)
                        </p>
                    </div>
                </div>
            </div>
            
            <!-- Second Quarter (Top Right) - Address -->
            <div class="quarter odd-top">
                <div class="col">
                    <div class="header">
                        <div class="logo">Σ</div>
                        <span class="header-title"> Ergo Vanity Wallet</span>
                    </div>
                    <div class="row">
                        <h1 class="title">Address</h1>
                    </div>
                    <div class="row flex-grow address-section">
                        <div class="bordered">
                            <p class="text-sm text-gray-600">Address /{3}</p>
                            <p class="key">
                                {0}
                            </p>
                            <div style="text-align: right;">
                                <!-- Smaller QR wrapper for the address quadrant -->
                                <div class="qr-wrapper qr-wrapper-small" style="display: inline-block;">
                                    {2}
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="row">
                        <p class="tip-text">
                            Use these addresses to receive ERG and tokens. You can share them safely.
                        </p>
                    </div>
                </div>
            </div>
            
            <!-- Third Quarter (Bottom Left) - Seed Phrase -->
            <div class="quarter even-bottom">
                <div class="col">
                    <div class="row">
                        <h1 class="title">Seed Phrase ({4}-word)</h1>
                    </div>
                    <div class="row flex-grow seed-phrase-section">
                        <div class="bordered flex-grow">
                            <p class="key" style="font-size: 16px; line-height: 1.6;">
                                {5}
                            </p>
                        </div>
                    </div>
                    <div class="row text-center">
                        <p style="text-align: center; font-size: 36px; color: #d9534f;">
                            ⚠️
                        </p>
                        <p style="text-align: center; color: #d9534f; font-weight: bold;">
                            WARNING: Keep this seed phrase secret!<br>
                            Anyone with access to it can access your funds.
                        </p>
                    </div>
                </div>
            </div>
            
            <!-- Fourth Quarter (Bottom Right) - Instructions -->
            <div class="quarter">
                <div class="col">
                    <div class="row">
                        <h1 class="title">Instructions</h1>
                    </div>
                    <div class="row flex-grow">
                        <div class="bordered">
                            <p class="key transform-rotate">
                                {6}
                            </p>
                            <p class="tip-text" style="margin-top: 10px; font-size: 11px;">
                                This text is deliberately flipped to confuse casual observers.
                            </p>
                        </div>
                    </div>
                    <div class="row">
                        <ul style="list-style-type: disc; padding-left: 25px;">
                            <li>Cut along the dotted lines and fold this paper wallet.</li>
                            <li>Store in a secure location safe from water, fire, and theft.</li>
                            <li>Make multiple copies and store them in different secure locations.</li>
                        </ul>
                    </div>
                </div>
            </div>
        </div>
    </div>
</body>
</html>"#,
        info.address,
        pseudo_pub_key,
        address_qr_svg,
        info.position,
        info.word_count,
        format_mnemonic(mnemonic),
        shuffled
    );

    // Always use .html extension
    let mut output_html_path = output_path.to_path_buf();
    output_html_path.set_extension("html");
    
    // Write the HTML file
    let mut file = File::create(&output_html_path).map_err(|e| e.to_string())?;
    file.write_all(html.as_bytes()).map_err(|e| e.to_string())?;
    
    println!("Paper wallet created: {}", output_html_path.display());
    Ok(())
}

/// Format mnemonic with spaces and line breaks
fn format_mnemonic(mnemonic: &str) -> String {
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    let word_count = words.len();
    
    // Format differently based on word count
    let words_per_line = if word_count == 12 {
        4  // 3 lines of 4 words for a 12-word mnemonic
    } else if word_count == 15 {
        5  // 3 lines of 5 words for a 15-word mnemonic
    } else {
        6  // 4 lines of 6 words for a 24-word mnemonic
    };
    
    // Group words by the calculated words per line
    let lines: Vec<String> = words.chunks(words_per_line)
        .map(|chunk| chunk.join(" "))
        .collect();
    
    // Join with HTML line breaks
    lines.join("<br>")
}

/// Create a reliable QR code SVG that works in HTML
fn create_reliable_qr_svg(data: &str) -> Result<String, String> {
    let qr = QrCode::new(data).map_err(|e| e.to_string())?;
    
    // Generate SVG using the proper renderer
    let svg = qr.render::<qrcode::render::svg::Color>()
        .min_dimensions(250, 250)
        .dark_color(qrcode::render::svg::Color("#000000"))
        .light_color(qrcode::render::svg::Color("#ffffff"))
        .build();
    
    Ok(svg)
}
