# SameSame Input Forwarder

Ein leistungsstarkes Tool zum Weiterleiten von Macbook-Tastatur- und Trackpad-Eingaben an einen Windows-Laptop über das Netzwerk.

## Features

- ✅ **Tastatur-Forwarding** mit German Layout Support
- ✅ **Modifier-Mapping**: Cmd → Strg, Option → Alt
- ✅ **Sonderzeichen**: Vollständige Unterstützung für ä, ö, ü, ß, +, -, ., , etc.
- ✅ **Maus/Trackpad-Bewegung**: Präzise Cursor-Steuerung
- ✅ **2-Finger-Scroll**: Automatisch invertiert für Windows (natural scrolling)
- ✅ **4-Finger-Gesten**: Desktop-Wechsel (Swipe left/right → Strg+Win+Left/Right)
- ✅ **Hotkey-Umschaltung**: Option+1 zum Wechseln zwischen macOS und Windows Modus
- ✅ **Input-Blocking**: Macbook-Eingaben werden blockiert, wenn im Windows-Modus

## Architektur

Das Projekt besteht aus drei Komponenten:

1. **protocol**: Shared Rust-Crate mit Event-Definitionen
2. **macos-client**: Tauri-basierte macOS-App zum Abfangen und Weiterleiten von Eingaben
3. **windows-server**: Rust-Binary zum Empfangen und Simulieren von Eingaben auf Windows

## Voraussetzungen

### macOS Client
- macOS 11.0 oder höher
- Xcode Command Line Tools
- Node.js 18+ und npm
- Rust 1.70+

### Windows Server
- Windows 10/11
- Rust 1.70+

## Installation & Build

### 1. Repository klonen

```bash
git clone <repository-url>
cd SameSame
```

### 2. macOS Client bauen

```bash
cd macos-client

# Dependencies installieren
npm install

# Development-Modus (zum Testen)
npm run tauri dev

# Production Build
npm run tauri build
```

Die fertige App befindet sich in `macos-client/src-tauri/target/release/bundle/macos/`.

### 3. Windows Server bauen

Auf einem Windows-Rechner oder mit Cross-Compilation:

```bash
cd windows-server

# Development Build
cargo build

# Production Build (optimiert)
cargo build --release
```

Das fertige Binary befindet sich in `target/release/samesame-windows-server.exe`.

## Nutzung

### Schritt 1: Windows Server starten

Auf dem Windows-Laptop:

```bash
# Navigiere zum Build-Verzeichnis
cd windows-server/target/release

# Starte den Server
./samesame-windows-server.exe
```

Der Server lauscht standardmäßig auf Port **24800** und wartet auf Verbindungen.

**Wichtig:** Stelle sicher, dass die Windows Firewall den Port 24800 für eingehende Verbindungen freigibt.

### Schritt 2: macOS Client starten

Auf dem Macbook:

1. Starte die SameSame App
2. Bei erstem Start: **Accessibility-Berechtigung erteilen**
   - Gehe zu Systemeinstellungen → Datenschutz & Sicherheit → Bedienungshilfen
   - Füge die SameSame App hinzu und aktiviere sie
3. Gib die IP-Adresse des Windows-Laptops ein (z.B. `192.168.1.100`)
4. Klicke auf **Connect**

### Schritt 3: Zwischen Modi wechseln

- **Option + 1** drücken, um zwischen macOS-Modus und Windows-Modus zu wechseln
- Im **macOS-Modus** (lila): Eingaben gehen normal an den Mac
- Im **Windows-Modus** (blau): Eingaben werden an Windows weitergeleitet und auf dem Mac blockiert

## Netzwerk-Konfiguration

### IP-Adresse des Windows-Laptops herausfinden

Auf Windows:

```cmd
ipconfig
```

Suche nach der IPv4-Adresse (z.B. `192.168.1.100`).

### Firewall konfigurieren (Windows)

1. Windows-Suche → "Windows Defender Firewall"
2. "Erweiterte Einstellungen"
3. "Eingehende Regeln" → "Neue Regel"
4. Port: **24800**
5. TCP-Protokoll
6. Verbindung zulassen

## Tastatur-Layout & Mapping

### Modifier-Keys

| macOS         | Windows      |
|---------------|--------------|
| Cmd (⌘)       | Strg         |
| Option (⌥)    | Alt          |
| Control (⌃)   | Win          |
| Shift (⇧)     | Shift        |

### Sonderzeichen (German Layout)

Alle deutschen Sonderzeichen werden korrekt übertragen:
- ä, ö, ü, ß
- Umlaute mit Shift: Ä, Ö, Ü
- Satzzeichen: +, -, ., ,, /, etc.

### Gesten-Mapping

| macOS Geste              | Windows Aktion                    |
|--------------------------|-----------------------------------|
| 4-Finger-Swipe Left      | Strg+Win+Left (Desktop wechseln)  |
| 4-Finger-Swipe Right     | Strg+Win+Right (Desktop wechseln) |
| 4-Finger-Swipe Up        | Win+Tab (Task View)               |
| 4-Finger-Swipe Down      | Win+D (Desktop anzeigen)          |
| 2-Finger-Scroll          | Scrollen (invertiert)             |

## Troubleshooting

### macOS Client startet nicht

**Problem:** "Accessibility permissions not granted"

**Lösung:**
1. Systemeinstellungen → Datenschutz & Sicherheit → Bedienungshilfen
2. SameSame App hinzufügen
3. App neu starten

### Verbindung schlägt fehl

**Problem:** "Failed to connect to server"

**Lösung:**
- Prüfe, ob Windows-Server läuft
- Prüfe die IP-Adresse (Windows: `ipconfig`, macOS: ping die IP)
- Prüfe Windows Firewall (Port 24800 muss offen sein)
- Prüfe, ob beide Geräte im selben Netzwerk sind

### Eingaben werden nicht weitergeleitet

**Problem:** Im Windows-Modus passiert nichts auf Windows

**Lösung:**
- Prüfe Verbindungsstatus in der App
- Prüfe Windows-Server-Logs
- Stelle sicher, dass du wirklich im Windows-Modus bist (blaues Icon)
- Drücke Option+1, um erneut zu wechseln

### Sonderzeichen funktionieren nicht

**Problem:** Deutsche Umlaute kommen falsch an

**Lösung:**
- Prüfe, ob Windows auch auf German Layout eingestellt ist
- Einige Sonderzeichen benötigen evtl. manuelle Anpassung im Key-Mapping

## Entwicklung

### Projekt-Struktur

```
SameSame/
├── Cargo.toml              # Workspace-Konfiguration
├── protocol/               # Shared Event-Definitionen
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── macos-client/           # Tauri macOS App
│   ├── package.json
│   ├── src/                # Frontend (HTML/CSS/JS)
│   └── src-tauri/          # Rust Backend
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── event_tap.rs    # CGEventTap für Input-Erfassung
│           ├── gestures.rs     # Gesten-Erkennung
│           ├── network.rs      # TCP-Client
│           └── state.rs        # App-State-Management
└── windows-server/         # Windows TCP-Server
    ├── Cargo.toml
    └── src/
        ├── main.rs
        └── input_simulator.rs  # SendInput API
```

### Debugging

#### macOS Client Logs

```bash
# Development-Modus mit Logs
RUST_LOG=debug npm run tauri dev
```

#### Windows Server Logs

```bash
# Mit Debug-Logs starten
set RUST_LOG=debug
./samesame-windows-server.exe
```

### Protokoll

Das Netzwerk-Protokoll verwendet **bincode** für effiziente Serialisierung:

- **TCP-Verbindung** auf Port 24800
- **Binäres Format** für niedrige Latenz
- **Message-Struktur**: `{ sequence: u64, event: InputEvent }`

## Bekannte Einschränkungen

- **Gesten-Erkennung**: Noch nicht vollständig implementiert (nur Swipe-Gesten)
- **Multi-Monitor**: Maus-Koordinaten sind für Single-Monitor optimiert
- **Latenz**: Ca. 20-50ms je nach Netzwerk-Qualität
- **Sicherheit**: Keine Verschlüsselung (nur für vertrauenswürdige Netzwerke)

## Zukünftige Features

- [ ] Verschlüsselte Verbindung (TLS)
- [ ] Multi-Monitor-Support
- [ ] Automatische Server-Erkennung (mDNS/Bonjour)
- [ ] Konfigurierbare Hotkeys
- [ ] Zoom-Gesten (Pinch-to-Zoom)
- [ ] Clipboard-Synchronisation

## Lizenz

MIT License - siehe LICENSE Datei

## Credits

Inspiriert von [Synergy](https://symless.com/synergy) und [Barrier](https://github.com/debauchee/barrier).

Built with:
- [Tauri](https://tauri.app) - Desktop App Framework
- [Rust](https://rust-lang.org) - Systems Programming Language
- [core-graphics](https://github.com/servo/core-graphics-rs) - macOS Event Taps
- [windows-rs](https://github.com/microsoft/windows-rs) - Windows API Bindings
