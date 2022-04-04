## json json files

| File            | Status | Case                                                 |
| --------------- | ------ | ---------------------------------------------------- |
| json_err_1.json | 403    | curl https://ip-api.com/json/24.48.0.1               |
| json_err_2.json | 200    | curl http://ip-api.com/json/24                       |
| json_err_3.json | 403    | curl 'https://pro.ip-api.com/json/24.48.0.1?key=foo' |

## batch json files

| File                            | Status | Case                                                                                                                                              |
| ------------------------------- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| batch_simple.json               | 200    | curl http://ip-api.com/batch -d '[{"query": "208.80.152.201", "fields": "city,country,countryCode,query", "lang": "ru"}, "8.8.8.8", "24.48.0.1"]' |
| batch_simple_with_part_err.json | 200    | curl http://ip-api.com/batch -d '["208.80.152.201", "2"]'                                                                                         |