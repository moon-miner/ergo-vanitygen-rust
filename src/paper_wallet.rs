use std::fs::File;
use std::io::Write;
use std::path::Path;
use qrcode::QrCode;
use chrono::Local;

/// Information for generating a paper wallet
pub struct PaperWalletInfo {
    pub address: String,
    pub mnemonic: String,
    pub word_count: usize,
    pub position: u32,
}

/// Options for wallet encryption
pub struct EncryptionOptions {
    pub encrypt_seed: bool,
    pub password_hint: Option<String>,
}

impl Default for EncryptionOptions {
    fn default() -> Self {
        Self {
            encrypt_seed: false,
            password_hint: None,
        }
    }
}

/// Generates a paper wallet HTML file with a fixed A4 layout.
/// The generated wallet has a bi-fold design with the seed phrase hidden when folded,
/// and detachable QR codes at the bottom of the page.
pub fn generate_paper_wallet(
    info: &PaperWalletInfo, 
    output_path: &Path,
    encryption_options: Option<EncryptionOptions>
) -> Result<(), String> {
    let encryption_options = encryption_options.unwrap_or_default();
    let address_qr = generate_qr_code(&info.address, 150)?;
    let small_qr = generate_qr_code(&info.address, 90)?;
    
    // Format the mnemonic for display (with numbered words)
    let formatted_mnemonic = format_mnemonic(&info.mnemonic, info.word_count);
    
    // Handle seed phrase QR code and encryption if needed
    let (seed_qr, encryption_message) = if encryption_options.encrypt_seed {
        let encrypted = encrypt_seed(&info.mnemonic)?;
        let hint = encryption_options.password_hint
            .map(|h| format!("\nHint: {}", h))
            .unwrap_or_default();
        let qr_data = format!("ENCRYPTED:{}{}", encrypted, hint);
        (
            generate_qr_code(&qr_data, 120)?,
            Some("This seed phrase is encrypted. Use your password to restore.")
        )
    } else {
        (generate_qr_code(&info.mnemonic, 120)?, None)
    };
    
    let current_date = Local::now().format("%Y-%m-%d").to_string();
    let short_address = format!("{}...{}", 
        &info.address[..8], 
        &info.address[info.address.len().saturating_sub(6)..]);
    
    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=210mm, initial-scale=1.0">
  <title>Ergo Paper Wallet</title>
  <style>
    /* ----- Reset and Base Styles ----- */
    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{
      font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      font-size: 11px;
      line-height: 1.3;
      color: #333;
      background: #f8f8f8;
    }}
    h1, h2, h3 {{ margin-bottom: 0.3em; font-weight: 600; }}
    h1 {{ font-size: 1.3rem; }}
    h2 {{ font-size: 1rem; }}
    h3 {{ font-size: 0.9rem; }}
    
    /* ----- Print Settings ----- */
    @page {{ size: A4 portrait; margin: 0; }}
    @media print {{
      body {{ background: #fff; }}
      .no-print {{ display: none !important; }}
    }}
    
    /* ----- Wallet Container ----- */
    .wallet-container {{
      width: 210mm;
      height: 297mm;
      margin: 1rem auto;
      background: white;
      box-shadow: 0 3px 10px rgba(0,0,0,0.15);
      position: relative;
      display: flex;
      flex-direction: column;
    }}
    
    /* ----- Main Wallet Section (Top 82%) ----- */
    .main-section {{
      display: grid;
      grid-template-columns: 1fr 1fr;
      grid-template-rows: 1fr 1fr;
      height: 82%;
    }}
    
    /* ----- Quadrant 1: Public Address (Top Left) ----- */
    .address-quadrant {{
      padding: 15px;
      border-right: 3px dashed #777;
      border-bottom: 3px dashed #777;
    }}
    
    /* ----- Quadrant 2: Instructions (Top Right) ----- */
    .instructions-quadrant {{
      padding: 15px;
      border-bottom: 3px dashed #777;
    }}
    
    /* ----- Quadrant 3: Seed Phrase (Bottom Left) ----- */
    .seed-quadrant {{
      padding: 15px;
      border-right: 3px dashed #777;
      background: #101010;
      background-image: 
        repeating-linear-gradient(45deg, #1a1a1a 0, #1a1a1a 2px, transparent 0, transparent 8px),
        repeating-linear-gradient(-45deg, #1a1a1a 0, #1a1a1a 2px, transparent 0, transparent 8px);
      color: white;
    }}
    
    /* ----- Quadrant 4: QR Codes (Bottom Right) ----- */
    .private-qr-quadrant {{
      padding: 15px;
      background: #101010;
      background-image: 
        repeating-linear-gradient(45deg, #1a1a1a 0, #1a1a1a 2px, transparent 0, transparent 8px),
        repeating-linear-gradient(-45deg, #1a1a1a 0, #1a1a1a 2px, transparent 0, transparent 8px);
      color: white;
      display: flex;
      flex-direction: column;
      justify-content: center;
      align-items: center;
    }}
    
    /* ----- Header Styles ----- */
    .header {{
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 15px;
    }}
    .logo {{
      display: flex;
      align-items: center;
    }}
    .logo-symbol {{
      font-size: 1.8em;
      font-weight: bold;
      color: #ff8c00;
      margin-right: 8px;
    }}
    
    /* ----- Address Styles ----- */
    .address-box {{
      background: #f9f9f9;
      border: 1px solid #e0e0e0;
      border-radius: 4px;
      padding: 8px;
      font-size: 9px;
      margin-bottom: 10px;
      word-break: break-all;
      font-family: monospace;
    }}
    .qr-container {{
      text-align: center;
      margin: 10px 0;
    }}
    .qr-label {{
      margin-top: 3px;
      font-size: 0.8em;
      color: #777;
    }}
    
    /* ----- Seed Section Styles ----- */
    .seed-warning {{
      display: flex;
      align-items: center;
      margin-bottom: 15px;
    }}
    .warning-icon {{
      font-size: 1.4em;
      color: #f39c12;
      margin-right: 10px;
    }}
    .seed-phrase {{
      background: rgba(255,255,255,0.05);
      border: 1px solid rgba(255,255,255,0.1);
      border-radius: 4px;
      padding: 10px;
      margin: 10px 0;
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 5px;
    }}
    .seed-word {{
      display: flex;
      align-items: center;
    }}
    .word-number {{
      color: #ff8c00;
      font-size: 0.65em;
      font-weight: bold;
      width: 16px;
      text-align: right;
      margin-right: 4px;
    }}
    .encryption-note {{
      background: rgba(243, 156, 18, 0.1);
      border: 1px solid rgba(243, 156, 18, 0.2);
      padding: 8px;
      border-radius: 4px;
      margin-bottom: 10px;
      text-align: center;
      font-size: 0.85em;
    }}
    .fold-arrows {{
      position: absolute;
      font-size: 16px;
      color: #777;
      transform: rotate(45deg);
    }}
    .fold-arrows.top-right {{
      top: 10px;
      right: 10px;
    }}
    .fold-arrows.bottom-left {{
      bottom: 10px;
      left: 10px;
    }}
    .fold-instructions {{
      position: absolute;
      background: #fef9c3;
      color: #854d0e;
      padding: 2px 8px;
      border-radius: 3px;
      font-size: 8px;
      white-space: nowrap;
      z-index: 10;
    }}
    .fold-instructions.vertical {{
      top: 40%;
      right: 3px;
      transform: translateY(-50%) rotate(90deg);
      transform-origin: right center;
    }}
    .fold-instructions.horizontal {{
      bottom: 3px;
      left: 50%;
      transform: translateX(-50%);
    }}
    .footer {{
      text-align: center;
      font-size: 9px;
      margin-top: 10px;
      color: rgba(255,255,255,0.6);
    }}
    
    /* ----- Detachable Cards Section (Bottom 18%) ----- */
    .cards-section {{
      height: 18%;
      border-top: 1px solid #ddd;
      padding: 5px 0;
      position: relative;
    }}
    .cards-divider {{
      position: absolute;
      top: 0;
      left: 0;
      width: 100%;
      text-align: center;
    }}
    .scissors-icon {{
      background: white;
      padding: 0 10px;
      position: relative;
      top: -8px;
      color: #777;
    }}
    .detachable-cards {{
      display: flex;
      justify-content: space-evenly;
      height: 100%;
      padding: 5px 10px;
    }}
    .qr-card {{
      border: 1px solid #ddd;
      border-radius: 5px;
      padding: 10px;
      width: 30%;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      position: relative;
    }}
    .qr-card::before {{
      content: "";
      position: absolute;
      top: -5px;
      left: 5px;
      right: 5px;
      height: 1px;
      border-top: 1px dashed #aaa;
    }}
    .qr-card-title {{
      font-weight: bold;
      margin-bottom: 5px;
      font-size: 9px;
      text-align: center;
    }}
    .qr-code-container {{
      width: 90px;
      height: 90px;
      margin: 0 auto;
      display: flex;
      justify-content: center;
      align-items: center;
      overflow: hidden;
    }}
    .qr-code-container svg {{
      max-width: 100%;
      max-height: 100%;
    }}
    .qr-card-address {{
      font-family: monospace;
      font-size: 7px;
      margin-top: 8px;
      text-align: center;
      word-break: break-all;
      width: 100%;
      overflow: hidden;
    }}
    
    /* ----- Buttons and Print Note ----- */
    .print-button {{
      position: fixed;
      top: 15px;
      right: 15px;
      padding: 8px 16px;
      background: #ff8c00;
      color: white;
      border: none;
      border-radius: 4px;
      font-weight: bold;
      font-size: 14px;
      cursor: pointer;
      box-shadow: 0 2px 5px rgba(0,0,0,0.1);
      z-index: 100;
    }}
    .print-button:hover {{ background: #e67e22; }}
    .print-note {{
      position: fixed;
      left: 15px;
      bottom: 15px;
      padding: 12px;
      background: #fffaeb;
      border: 1px solid #ffeaa7;
      border-radius: 6px;
      width: 280px;
      font-size: 12px;
      line-height: 1.4;
      box-shadow: 0 2px 4px rgba(0,0,0,0.05);
      z-index: 100;
    }}
    .fold-diagram {{
      margin-top: 8px;
      padding: 5px;
      background: white;
      border-radius: 3px;
    }}
    .fold-diagram-inner {{
      width: 100%;
      height: 80px;
      border: 1px solid #ddd;
      position: relative;
      font-size: 9px;
      display: grid;
      grid-template-columns: 1fr 1fr;
      grid-template-rows: 1fr 1fr;
    }}
    .fold-diagram-quadrant {{
      border: 1px solid #eee;
      display: flex;
      justify-content: center;
      align-items: center;
      position: relative;
    }}
    .fold-diagram-q1 {{
      border-right: 2px dashed #777;
      border-bottom: 2px dashed #777;
    }}
    .fold-diagram-q2 {{
      border-bottom: 2px dashed #777;
    }}
    .fold-diagram-q3 {{
      border-right: 2px dashed #777;
      background: #eee;
    }}
    .fold-diagram-q4 {{
      background: #eee;
    }}
    .cut-line {{
      position: absolute;
      bottom: -12px;
      left: 0;
      width: 100%;
      border-top: 1px solid #777;
    }}
    .fold-step {{
      position: absolute;
      font-size: 8px;
      padding: 1px 3px;
      background: white;
      border: 1px solid #ddd;
      border-radius: 50%;
    }}
    .fold-step-1 {{
      right: -5px;
      top: 40%;
    }}
    .fold-step-2 {{
      bottom: -5px;
      left: 40%;
    }}
    .fold-step-3 {{
      left: 10px;
      bottom: -25px;
    }}
  </style>
</head>
<body>
  <button class="print-button no-print" onclick="window.print()">Print Wallet</button>
  <div class="print-note no-print">
    <strong>Quad-Fold Wallet Instructions:</strong><br>
    • Use thick, high-quality paper<br>
    • Print at 100% scale (no scaling)<br>
    • Fold along the dashed lines in numbered order<br>
    • Cut along the bottom edge to detach QR cards
    <div class="fold-diagram">
      <div class="fold-diagram-inner">
        <div class="fold-diagram-quadrant fold-diagram-q1">Address</div>
        <div class="fold-diagram-quadrant fold-diagram-q2">Instructions</div>
        <div class="fold-diagram-quadrant fold-diagram-q3">Seed Phrase</div>
        <div class="fold-diagram-quadrant fold-diagram-q4">Private QR</div>
        <div class="fold-step fold-step-1">1</div>
        <div class="fold-step fold-step-2">2</div>
        <div class="cut-line"></div>
        <div class="fold-step fold-step-3">3</div>
      </div>
    </div>
  </div>
  
  <div class="wallet-container">
    <!-- Main Wallet Section with Quad-Fold Design -->
    <div class="main-section">
      <!-- Quadrant 1: Public Address (Top Left - Visible when folded) -->
      <div class="address-quadrant">
        <div class="header">
          <div class="logo">
            <span class="logo-symbol">Σ</span>
            <div>
              <h1>Ergo Paper Wallet</h1>
              <div style="font-size: 0.8em; color: #666;">Cold Storage • {date}</div>
            </div>
          </div>
        </div>
        
        <h2>Ergo Address</h2>
        <div class="address-box">
          {address}
        </div>
        <div class="qr-container">
          {address_qr}
          <div class="qr-label">Scan to receive funds</div>
        </div>
        <div style="font-size: 0.8em; color: #666; margin-top: 5px;">
          {word_count}-word seed • Path: m/44'/429'/0'/0/{position}
        </div>
        
        <div class="fold-instructions vertical">FOLD ALONG DASHED LINE</div>
      </div>
      
      <!-- Quadrant 2: Instructions (Top Right - Visible when folded) -->
      <div class="instructions-quadrant">
        <h2>Wallet Instructions</h2>
        <ol style="margin-left: 16px;">
          <li>Print on high-quality paper</li>
          <li>Fold along both dashed lines in order (1, 2)</li>
          <li>Cut along bottom edge to detach QR cards (3)</li>
          <li>Keep this document in a safe place</li>
          <li>Never share your seed phrase with anyone</li>
        </ol>
        
        <div style="margin-top: 15px;">
          <h3>Recovery Instructions</h3>
          <div style="font-size: 0.9em;">
            1. Use a compatible Ergo wallet app<br>
            2. Select "Restore Wallet"<br>
            3. Enter the exact seed phrase from inside<br>
            4. Verify the address matches this wallet
          </div>
        </div>
        
        <div style="margin-top: 15px;">
          <h3>Security Tips</h3>
          <ul style="margin-left: 16px;">
            <li>Consider metal backup for long-term storage</li>
            <li>Test recovery before storing large amounts</li>
            <li>Keep multiple copies in different secure locations</li>
          </ul>
        </div>
        
        <div class="fold-arrows top-right">↘</div>
      </div>
      
      <!-- Quadrant 3: Seed Phrase (Bottom Left - Hidden when folded) -->
      <div class="seed-quadrant">
        <div class="seed-warning">
          <span class="warning-icon">⚠️</span>
          <div>
            <div style="font-weight: bold; color: #f39c12;">KEEP YOUR SEED PHRASE SECRET!</div>
            <div style="font-size: 0.8em;">Anyone with these words can access and steal your funds</div>
          </div>
        </div>
        
        <h2>Secret Recovery Phrase</h2>
        {encryption_message}
        <div class="seed-phrase">
          {mnemonic}
        </div>
        
        <div class="fold-instructions horizontal">FOLD ALONG DASHED LINE</div>
        <div class="fold-arrows bottom-left">↗</div>
      </div>
      
      <!-- Quadrant 4: Private QR (Bottom Right - Hidden when folded) -->
      <div class="private-qr-quadrant">
        <div class="qr-container">
          {seed_qr}
          <div class="qr-label" style="color: #f39c12;">PRIVATE: Scan to import wallet</div>
        </div>
        
        <div class="footer" style="margin-top: 20px;">
          <div>Generated with Ergo Vanitygen • Use at your own risk</div>
          <div>ergoplatform.org</div>
        </div>
      </div>
    </div>
    
    <!-- Detachable QR Cards Section -->
    <div class="cards-section">
      <div class="cards-divider">
        <span class="scissors-icon">✂️ CUT HERE ✂️</span>
      </div>
      <div class="detachable-cards">
        <div class="qr-card">
          <div class="qr-card-title">ERGO WALLET</div>
          <div class="qr-code-container">
            {small_qr}
          </div>
          <div class="qr-card-address">{short_address}</div>
        </div>
        
        <div class="qr-card">
          <div class="qr-card-title">ERGO WALLET</div>
          <div class="qr-code-container">
            {small_qr}
          </div>
          <div class="qr-card-address">{short_address}</div>
        </div>
        
        <div class="qr-card">
          <div class="qr-card-title">ERGO WALLET</div>
          <div class="qr-code-container">
            {small_qr}
          </div>
          <div class="qr-card-address">{short_address}</div>
        </div>
      </div>
    </div>
  </div>
</body>
</html>"#,
        date = current_date,
        address = info.address,
        position = info.position,
        word_count = info.word_count,
        address_qr = address_qr,
        seed_qr = seed_qr,
        small_qr = small_qr,
        short_address = short_address,
        mnemonic = formatted_mnemonic,
        encryption_message = encryption_message
          .map(|msg| format!(r#"<div class="encryption-note">{}</div>"#, msg))
          .unwrap_or_default()
    );
    
    let mut output_path = output_path.to_path_buf();
    output_path.set_extension("html");
    let mut file = File::create(&output_path).map_err(|e| e.to_string())?;
    file.write_all(html.as_bytes()).map_err(|e| e.to_string())?;
    
    println!("Paper wallet created: {}", output_path.display());
    Ok(())
}

/// Formats the mnemonic phrase with numbered words
fn format_mnemonic(mnemonic: &str, _word_count: usize) -> String {
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    let word_elements: Vec<String> = words.iter().enumerate().map(|(i, word)| {
        format!(
            r#"<div class="seed-word">
                <span class="word-number">{:02}.</span>
                <span style="font-family: monospace;">{}</span>
              </div>"#,
            i + 1, word
        )
    }).collect();
    word_elements.join("\n")
}

/// Generates a QR code in SVG format
fn generate_qr_code(data: &str, size: u32) -> Result<String, String> {
    let qr = QrCode::new(data.as_bytes()).map_err(|e| e.to_string())?;
    let svg = qr.render::<qrcode::render::svg::Color>()
        .min_dimensions(size, size)
        .quiet_zone(true)
        .dark_color(qrcode::render::svg::Color("#000000"))
        .light_color(qrcode::render::svg::Color("#ffffff"))
        .build();
    Ok(svg)
}

/// Simple XOR-based encryption (for obfuscation only) for seed phrases
fn encrypt_seed(seed: &str) -> Result<String, String> {
    println!("Enter encryption password for paper wallet (not stored):");
    let password = rpassword::read_password().map_err(|e| e.to_string())?;
    if password.is_empty() {
        return Ok(seed.to_string());
    }
    let mut encrypted = String::with_capacity(seed.len() * 2);
    let password_bytes: Vec<u8> = password.bytes().collect();
    for (i, byte) in seed.bytes().enumerate() {
        let key_byte = password_bytes[i % password_bytes.len()];
        let encrypted_byte = byte ^ key_byte;
        encrypted.push_str(&format!("{:02x}", encrypted_byte));
    }
    Ok(encrypted)
}
