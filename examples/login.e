// Login con retry e fallback locale
do
    with browser do
        open "https://example.com/login"
        with page { timeout: 5s } do
            retry 3 times do
                login "mario" "password123"
                wait until visible ".dashboard"
            done or do
                log "login fallito"
                stop
            done
        done
    done
done
