# pattern-clock
(ongoing)

## Dependencies

- dioxus
- ractor
- burn

API Tests in folder `bruno/`

## Serving App

```sh
# ===============================================
# Ubuntu
# ===============================================
# Launches Desktop and Web
./serve_all.sh

# Web server http://127.0.0.1:8080
dx serve
# Desktop app
dx serve --platform desktop
# ===============================================
```

Both will use the same server backend for API endpoints. The web interface will be available at http://127.0.0.1:8080, and the desktop app will open in a separate window.
