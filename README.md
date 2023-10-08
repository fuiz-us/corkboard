# Corkboard

Service to store images temporarily.

## REST API

### Upload

`POST -F image=bytes /upload`

Ok Response: `"MediaID"`. The image stays availble for an hour.

### Retrieve

`GET /upload/{media_id}`

It responds with bytes of content-type: `image/png`.

### Exists

`GET /exists/{media_id}`

`true` if the image exists and `false` if it doesn't.