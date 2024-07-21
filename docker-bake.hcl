group "default" {
  targets = ["telegram-bot-biostatus"]
}

target "telegram-bot-biostatus" {
  inherits = ["docker-metadata-action-telegram-bot-biostatus"]
  dockerfile = "Dockerfile"
  target = "telegram-bot-biostatus"
}

# Targets to allow injecting customizations from Github Actions.

target "docker-metadata-action-telegram-bot-biostatus" {}
