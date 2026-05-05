// connect.g — import last 30 activities
time every week at 03:00 do
    with browser do
        open "https://connect.g.com"

        with page { timeout: 15s } do
            login "user@email.com" "password" or do
                log "login failed"
                stop
            done

            open "https://connect.g.com/modern/activities"
            wait until visible ".activities-container"

            retry 5 times do
                find ".load-more-btn"
                when item visible do
                    click
                done
            done

            find all ".activity-card"

            when count >= 30 do
                click "#export-all-btn"
                wait download
            done
        done
    done

    log "import complete"
    email to "admin@example.com" file "export.zip"
done or log error
