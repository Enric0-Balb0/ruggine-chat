# Manuale Utente - Ruggine Chat

## Indice
1. [Introduzione](#1-introduzione)
2. [Requisiti di Sistema](#2-requisiti-di-sistema)
3. [Installazione](#3-installazione)
4. [Primo Avvio e Registrazione](#4-primo-avvio-e-registrazione)
5. [Interfaccia Utente](#5-interfaccia-utente)
6. [Funzionalità Principali](#6-funzionalità-principali)
7. [Gestione Gruppi](#7-gestione-gruppi)
8. [Inviti e Partecipazione](#8-inviti-e-partecipazione)
9. [Gestione Profilo Utente](#9-gestione-profilo-utente)
10. [Monitoraggio Sistema (CPU Logs)](#10-monitoraggio-sistema-cpu-logs)
11. [Appendice: Informazioni Tecniche](#appendice-informazioni-tecniche)

---

## 1. Introduzione

**Ruggine Chat** è un'applicazione di messaggistica istantanea moderna e sicura, progettata per comunicazioni di gruppo in tempo reale. L'applicazione offre:

- ✅ Chat di gruppo in tempo reale
- ✅ Interfaccia utente intuitiva e responsive
- ✅ Tema chiaro/scuro personalizzabile
- ✅ Sistema di inviti per nuovi membri
- ✅ Sincronizzazione cross-platform
- ✅ Sicurezza e privacy avanzate

---

## 2. Requisiti di Sistema

### Windows
- **Sistema Operativo**: Windows 10 (versione 1903 o superiore) o Windows 11
- **Architettura**: x64 (64-bit)
- **RAM**: Minimo 4 GB
- **Spazio disco**: 100 MB disponibili
- **Connessione**: Internet richiesta per il funzionamento

### macOS (Supporto futuro)
- **Sistema Operativo**: macOS 10.13 (High Sierra) o superiore
- **Architettura**: Intel x64 o Apple Silicon (M1/M2)
- **RAM**: Minimo 4 GB
- **Spazio disco**: 100 MB disponibili

### Linux (Supporto futuro)
- **Distribuzioni supportate**: Ubuntu 18.04+, Debian 10+, Fedora 32+
- **Architettura**: x64 (64-bit)
- **RAM**: Minimo 4 GB
- **Spazio disco**: 100 MB disponibili

---

## 3. Installazione

### 3.1 Download dell'Installer

1. Scarica l'installer appropriato per il tuo sistema dalla sezione release del progetto
2. Per Windows sono disponibili due opzioni:
   - **MSI Installer** (`Ruggine Chat_x.x.x_x64_en-US.msi`) - Raccomandato per installazioni aziendali
   - **NSIS Installer** (`Ruggine Chat_x.x.x_x64-setup.exe`) - Installer semplificato

### 3.2 Installazione su Windows

#### Usando MSI Installer:
1. Fai doppio clic sul file `.msi` scaricato
2. Segui la procedura guidata di installazione
3. Accetta i termini di licenza
4. Scegli la cartella di destinazione (predefinita: `C:\Program Files\Ruggine Chat`)
5. Clicca su "Installa"
6. Al termine, l'applicazione sarà disponibile nel Menu Start

#### Usando NSIS Installer:
1. Fai doppio clic sul file `-setup.exe` scaricato
2. Se Windows mostra un avviso di sicurezza, clicca su "Ulteriori informazioni" e poi "Esegui comunque"
3. Segui la procedura guidata:
   - Benvenuto nell'installazione
   - Accettazione licenza
   - Scelta cartella di installazione
   - Selezione componenti (collegamento desktop, menu Start)
4. Clicca su "Installa"
5. Al termine dell'installazione, puoi scegliere di avviare immediatamente l'applicazione

### 3.3 Verifica dell'Installazione

Dopo l'installazione:
1. Cerca "Ruggine Chat" nel Menu Start di Windows
2. Oppure utilizza il collegamento sul desktop (se creato)
3. Al primo avvio, l'applicazione dovrebbe aprirsi senza errori

---

## 4. Primo Avvio e Registrazione

### 4.1 Configurazione Iniziale

Al primo avvio di Ruggine Chat:

1. **Schermata di Benvenuto**: Si aprirà la pagina di login/registrazione
2. **Configurazione Server**: L'applicazione si connetterà automaticamente al server configurato

### 4.2 Registrazione Nuovo Account

![Schermata di Registrazione](./screenshots/01-registrazione.png)
*Figura 1: Schermata di registrazione nuovo account*

1. Nella schermata iniziale, clicca su **"Registrati"**
2. Compila il modulo di registrazione:
   - **Nome utente**: Scegli un nome utente unico (3-50 caratteri)
   - **Email**: Inserisci un indirizzo email valido
   - **Password**: Crea una password sicura (minimo 8 caratteri)
   - **Conferma Password**: Ripeti la password per conferma
3. Clicca su **"Registrati"**
4. Se i dati sono corretti, sarai reindirizzato alla dashboard principale

### 4.3 Login con Account Esistente

![Schermata di Login](./screenshots/02-login.png)
*Figura 2: Schermata di login con account esistente*

1. Nella schermata iniziale, inserisci:
   - **Email**: Il tuo indirizzo email registrato
   - **Password**: La tua password
2. Clicca su **"Accedi"**
3. Se le credenziali sono corrette, accederai alla dashboard

### 4.4 Gestione Errori di Login

Se riscontri problemi durante il login:
- **Credenziali errate**: Verifica email e password
- **Account non trovato**: Assicurati di essere registrato o procedi con la registrazione
- **Problemi di connessione**: Controlla la tua connessione internet

---

## 5. Interfaccia Utente

### 5.1 Layout Principale

![Dashboard Principale](./screenshots/03-dashboard-principale.png)
*Figura 3: Dashboard principale di Ruggine Chat*

L'interfaccia di Ruggine Chat è divisa in sezioni principali:

```
┌─────────────────────────────────────────────────────────┐
│ [Logo] Ruggine Chat           [Tema] [Profilo] [Logout] │ ← Header
├─────────────────┬───────────────────────────────────────┤
│                 │                                       │
│   Lista Gruppi  │           Area Chat Principale        │ ← Contenuto
│                 │                                       │
│  - Gruppo 1     │  ┌─────────────────────────────────┐  │
│  - Gruppo 2     │  │        Messaggi Chat            │  │
│  - Gruppo 3     │  │                                 │  │
│                 │  └─────────────────────────────────┘  │
│                 │  [Input Messaggio...] [Invia]         │
├─────────────────┴───────────────────────────────────────┤
│              Footer / Stato Connessione                 │ ← Footer
└─────────────────────────────────────────────────────────┘
```

### 5.2 Componenti dell'Interfaccia

#### Header (Barra Superiore)
- **Logo Ruggine Chat**: Torna alla dashboard principale
- **Toggle Tema**: Passa tra tema chiaro e scuro
- **Menu Profilo**: Accesso alle impostazioni utente
- **Logout**: Esci dall'applicazione

#### Sidebar Sinistra (Lista Gruppi)
- **Lista Gruppi**: Tutti i gruppi di cui fai parte
- **Indicatori Messaggi Non Letti**: Badge numerici sui gruppi con nuovi messaggi
- **Pulsante Nuovo Gruppo**: Crea un nuovo gruppo (se disponibile)

#### Area Centrale (Chat)
- **Header Chat**: Nome del gruppo selezionato e numero partecipanti
- **Area Messaggi**: Cronologia messaggi del gruppo
- **Input Messaggio**: Campo per scrivere nuovi messaggi
- **Pulsante Invio**: Invia il messaggio

---

## 6. Funzionalità Principali

### 6.1 Invio Messaggi

![Area Chat Attiva](./screenshots/04-chat-area.png)
*Figura 4: Area chat con conversazione attiva*

1. **Seleziona un Gruppo**: Clicca su un gruppo nella sidebar sinistra
2. **Scrivi il Messaggio**: Digita il tuo messaggio nell'area di input in basso
3. **Invia**: 
   - Premi **Enter** sulla tastiera, oppure
   - Clicca sul pulsante **"Invia"**
4. Il messaggio apparirà immediatamente nella chat

#### Caratteristiche dei Messaggi:
- **Lunghezza massima**: 1000 caratteri per messaggio
- **Formattazione**: Supporto per testo semplice
- **Timestamp**: Ogni messaggio mostra data e ora di invio
- **Autore**: Il nome dell'utente che ha inviato il messaggio

### 6.2 Ricezione Messaggi

- I **messaggi in tempo reale** appaiono automaticamente
- I **messaggi non letti** sono evidenziati con badge numerici sui gruppi
- **Scroll automatico**: La chat scorre automaticamente ai nuovi messaggi
- **Notifiche**: (Se abilitate) riceverai notifiche per nuovi messaggi

### 6.3 Cambio Tema

![Confronto Temi](./screenshots/05-temi-confronto.png)
*Figura 5: Confronto tra tema chiaro (sinistra) e tema scuro (destra)*

1. Clicca sull'icona **tema** (sole/luna) nell'header
2. L'interfaccia passerà automaticamente tra:
   - **Tema Chiaro**: Sfondo bianco, testo scuro
   - **Tema Scuro**: Sfondo scuro, testo chiaro
3. La preferenza viene salvata automaticamente

---

## 7. Gestione Gruppi

### 7.1 Visualizzazione Gruppi

- **Lista Gruppi**: Tutti i tuoi gruppi sono elencati nella sidebar sinistra
- **Gruppo Attivo**: Il gruppo attualmente selezionato è evidenziato
- **Indicatori Attività**: Badge con numero di messaggi non letti

### 7.2 Creazione Nuovo Gruppo

![Modal Creazione Gruppo](./screenshots/06-crea-gruppo.png)
*Figura 6: Modal per la creazione di un nuovo gruppo*

1. Clicca sul pulsante **"+ Nuovo Gruppo"** (se disponibile)
2. Compila il modulo:
   - **Nome Gruppo**: Scegli un nome descrittivo (max 256 caratteri)
   - **Descrizione**: (Opzionale) Breve descrizione del gruppo (max 1024 caratteri)
3. Clicca su **"Crea Gruppo"**
4. Il nuovo gruppo apparirà nella tua lista

### 7.3 Impostazioni Gruppo

![Impostazioni Gruppo](./screenshots/07-impostazioni-gruppo.png)
*Figura 7: Pannello impostazioni gruppo con lista membri*

Per accedere alle impostazioni di un gruppo:
1. Seleziona il gruppo
2. Clicca sull'icona **impostazioni** (ingranaggio) nell'header della chat
3. Potrai vedere:
   - **Informazioni Gruppo**: Nome, descrizione, data creazione
   - **Lista Membri**: Tutti i partecipanti del gruppo
   - **Gestione Membri**: (Solo amministratori) Invita o rimuovi membri

---

## 8. Inviti e Partecipazione

### 8.1 Panoramica Sistema Inviti

Il sistema di inviti di Ruggine Chat permette di gestire l'accesso ai gruppi in modo controllato. Gli inviti possono essere inviati solo dagli amministratori dei gruppi e devono essere accettati dai destinatari per diventare effettivi.

### 8.2 Ricevere Inviti

![Lista Inviti Ricevuti](./screenshots/08-inviti-ricevuti.png)
*Figura 8: Schermata con la lista degli inviti ricevuti*

Quando ricevi un invito a un gruppo:
1. **Notifica Invito**: Riceverai una notifica di nuovo invito
2. **Accesso agli Inviti**: Clicca sul menu profilo e seleziona "Inviti"
3. **Visualizzazione Inviti**: Potrai vedere tutti gli inviti che hai ricevuto con:
   - Nome del gruppo che ti ha invitato
   - Chi ha inviato l'invito (amministratore)
   - Data e ora dell'invito
   - Ruolo che ti è stato assegnato nel gruppo
   - Stato dell'invito (Pending, Accepted, Rejected)

#### Gestire Inviti Ricevuti:

**Accettare un Invito:**
1. Nella lista inviti, trova l'invito da accettare
2. Clicca sul pulsante **"Accetta"**
3. Il gruppo verrà aggiunto alla tua lista gruppi
4. Potrai immediatamente partecipare alle conversazioni
5. L'invito cambierà stato in "Accepted"

**Rifiutare un Invito:**
1. Nella lista inviti, trova l'invito da rifiutare
2. Clicca sul pulsante **"Rifiuta"**
3. L'invito cambierà stato in "Rejected"
4. Non sarai aggiunto al gruppo

### 8.3 Invitare Nuovi Membri (Solo Amministratori)

Se sei amministratore di un gruppo, puoi invitare nuovi membri:

#### Inviare Nuovi Inviti:
![Modal Invita Membro](./screenshots/09-invita-membro.png)
*Figura 9: Modal per invitare un nuovo membro al gruppo*

1. Vai alle **impostazioni del gruppo** o usa il pulsante "Nuovo Invito"
2. Compila il modulo:
   - **Nome Utente**: Inserisci il nome utente esatto della persona da invitare
   - **Gruppo**: Seleziona il gruppo (se amministri più gruppi)
   - **Ruolo**: Scegli tra "Member" o "Admin"
3. Clicca su **"Invia Invito"**
4. L'utente riceverà l'invito nella sua lista


---

## 9. Gestione Profilo Utente

### 9.1 Accesso alla Pagina Profilo

![Pagina Profilo](./screenshots/10-pagina-profilo.png)
*Figura 10: Pagina profilo utente con informazioni e impostazioni*

Per accedere al tuo profilo:
1. Clicca sul **menu profilo** nell'header (icona utente)
2. Seleziona **"Profilo utente"** dal menu a tendina
3. Si aprirà la pagina dedicata al tuo profilo

### 9.2 Informazioni Profilo

Nella pagina profilo puoi visualizzare e gestire:

#### Informazioni Base:
- **Nome Utente**: Il tuo identificativo univoco nel sistema
- **Email**: L'indirizzo email associato al tuo account
- **Data Registrazione**: Quando ti sei registrato al servizio
- **Ultimo Accesso**: Data e ora dell'ultimo login

#### Avatar e Presentazione:
- **Foto Profilo**: (Se supportato) La tua immagine profilo
- **Stato**: Il tuo stato attuale (Online, Assente, Non Disturbare)
- **Bio**: (Se supportato) Una breve descrizione di te
---

## 10. Monitoraggio Sistema (CPU Logs)

### 10.1 Accesso ai CPU Logs (Solo Amministratori)

![Dashboard CPU Logs](./screenshots/11-cpu-logs-dashboard.png)
*Figura 12: Dashboard principale dei CPU Logs con grafici e metriche*

I CPU Logs sono disponibili solo per gli utenti con privilegi di amministratore:
1. Clicca sul **menu profilo** nell'header
2. Se sei amministratore, vedrai l'opzione **"CPU Logs"**
3. Clicca per accedere al dashboard di monitoraggio

### 10.2 Panoramica Dashboard

Il dashboard CPU Logs fornisce informazioni dettagliate sulle prestazioni del server:

#### Metriche Principali:
- **CPU Usage**: Percentuale di utilizzo CPU in tempo reale

---

