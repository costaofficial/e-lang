// Scarica file e processali
time at 18:00 do
    with browser do
        open "https://reportingsito.com"
        with page do
            login "user" "pass"
            click "#export-btn"
            wait download
        done
    done

    wait until visible ".loading"
    watch "downloads/" do
        with file "*.csv" do
            upload to "https://api.import.com/csv"
            log "importato csv"
        done
    done
done
