# Guida alla Personalizzazione Installer

Questa cartella contiene i file per personalizzare gli installer di Ruggine Chat su tutte le piattaforme.

## File di Configurazione

### Windows (WiX e NSIS)
- `wix-banner.bmp`: Banner per l'installer WiX (493x58 pixel)
- `wix-dialog.bmp`: Immagine di dialogo WiX (493x312 pixel)
- `nsis-header.bmp`: Header per l'installer NSIS (150x57 pixel)
- `nsis-sidebar.bmp`: Sidebar per l'installer NSIS (164x314 pixel)
- `nsis-italian.nsh`: Traduzioni italiane per NSIS
- `installer-hooks.nsh`: Hook personalizzati per NSIS

### macOS
- `dmg-background.png`: Sfondo per il file DMG (660x400 pixel)

### Linux
- `ruggine-chat.desktop`: File desktop entry per l'integrazione sistema

## Come Aggiungere le Immagini

1. **Banner WiX** (493x58px, BMP):
   - Crea un'immagine con il logo Ruggine Chat
   - Usa colori che si adattino al tema dell'app
   - Salva come `wix-banner.bmp`

2. **Dialog WiX** (493x312px, BMP):
   - Immagine principale del dialogo di installazione
   - Può includere screenshot dell'app o artwork
   - Salva come `wix-dialog.bmp`

3. **Header NSIS** (150x57px, BMP):
   - Piccolo header per l'installer NSIS
   - Logo compatto o nome app
   - Salva come `nsis-header.bmp`

4. **Sidebar NSIS** (164x314px, BMP):
   - Immagine laterale dell'installer
   - Design verticale con brand identity
   - Salva come `nsis-sidebar.bmp`

5. **Background DMG** (660x400px, PNG):
   - Sfondo per il finder del DMG su macOS
   - Design pulito con indicazioni visive
   - Salva come `dmg-background.png`

## Note

- Tutte le immagini devono essere ottimizzate per dimensioni ridotte
- I colori dovrebbero essere coerenti con il brand Ruggine Chat
- Le immagini BMP devono essere in formato 24-bit senza compressione
- La configurazione è già impostata in `tauri.conf.json`
