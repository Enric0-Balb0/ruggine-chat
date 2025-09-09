; Hook personalizzati per l'installer NSIS
; Questi hook vengono eseguiti durante l'installazione

Function .onInit
    ; Controlla se l'applicazione è già in esecuzione
    FindWindow $0 "" "Ruggine Chat"
    StrCmp $0 0 notRunning
        MessageBox MB_OK|MB_ICONEXCLAMATION "Ruggine Chat è attualmente in esecuzione. Chiudi l'applicazione prima di continuare l'installazione."
        Abort
    notRunning:
        
    ; Imposta la lingua italiana come predefinita
    StrCpy $LANGUAGE ${LANG_ITALIAN}
FunctionEnd

Function .onInstSuccess
    ; Mostra messaggio di successo
    MessageBox MB_OK "Installazione completata con successo!$\n$\nRuggine Chat è ora pronto per l'uso."
FunctionEnd

Function un.onInit
    ; Conferma prima della disinstallazione
    MessageBox MB_ICONQUESTION|MB_YESNO|MB_DEFBUTTON2 "Sei sicuro di voler rimuovere Ruggine Chat?" IDYES +2
    Abort
FunctionEnd
