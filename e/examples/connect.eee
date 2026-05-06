// Weekly report: export + process + notify
time every day at 02:00 do
    // 1. Extract data
    with browser do
        open "https://connect.api"
        with page { timeout: 10s } do
            login "user" "password" or stop
            click "#export-all"
            wait download
        done
    done

    // 2. Process
    watch "downloads/" do
        with file "*.fit" do
            upload to "https://fitness.db/import"
            log "fit imported"
        done
    done

    // 3. Notify
    email to "admin@example.com" file "report.pdf"
done or log error
