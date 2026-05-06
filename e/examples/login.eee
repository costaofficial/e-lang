// Login with retry and local fallback
do
    with browser do
        open "https://example.com/login"
        with page { timeout: 5s } do
            retry 3 times do
                login "mario" "password123"
                wait until visible ".dashboard"
            done or do
                log "login failed"
                stop
            done
        done
    done
done
