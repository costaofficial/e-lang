// Backup ogni ora
time every hour at 00 do
    with file "backup.sql" do
        write file "backup.sql" "dump completo"
    done
    email to "admin@esempio.com" file "backup.sql"
done or log error
