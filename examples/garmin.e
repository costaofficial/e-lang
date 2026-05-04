// Automazione completa: report settimanale
time every day at 02:00 do
    // 1. Estrai dati
    with browser do
        open "https://connect.garmin.com"
        with page { timeout: 10s } do
            login "user" "password" or stop
            click "#export-all"
            wait download
        done
    done

    // 2. Processa
    watch "downloads/" do
        with file "*.fit" do
            upload to "https://fitness.db/import"
            log "fit importato"
        done
    done

    // 3. Notifica
    email to "admin@esempio.com" file "report.pdf"
done or log error
