// Caffè alle 9:00 ogni mattina
time every day at 09:00 do
    with browser do
        open "https://google.com"
        with page do
            find "#search"
            click "q"
            log "cercato caffe"
        done
    done
done or log error
