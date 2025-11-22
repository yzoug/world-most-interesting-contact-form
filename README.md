# E2EE-encrypted and hardened contact form

This project contains what's needed to deploy a hardened contact form backend, protected by Traefik and CrowdSec, and coded in Rust with Axum.

It is meant to be used with a contact form and Javascript code to encrypt the payload using OpenPGP. You can [read the article here](https://zoug.fr/world-most-interesting-contact-form/) to see it in action, and for more info on how to set everything up.

The Javascript code used in the article is available on [my website's repo](https://github.com/yzoug/zougfr/blob/main/static/pgp-contact.js).
