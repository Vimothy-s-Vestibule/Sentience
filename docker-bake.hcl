variable "REGISTRY" {
  default = ""
}

group "default" {
  targets = [
    "frontend",
    "backend"
  ]
}

target "frontend" {
    dockerfile = "Dockerfile.frontend"
    tags       = ["${REGISTRY}thavlik/sentience-frontend:latest"]
    push       = true
}

target "backend" {
    dockerfile = "Dockerfile.backend"
    tags       = ["${REGISTRY}thavlik/sentience-backend:latest"]
    push       = true
}