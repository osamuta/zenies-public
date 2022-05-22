# data transferer
1. check sqlite file integrity
2. if sqlite file is broken, export data to csv.
3. create new sqlite file, import csv of step 4 to new file.


cargo run -p data_transferer --release --  mongodb://192.168.2.101 ticker 2021-09-03T00:00:00+09:00 newest