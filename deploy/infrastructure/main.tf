module service {
  source = "github.com/aelred/provision//modules/service"
  name = "skakoui"
}

output next_steps {
  value = module.service.next_steps
}

output dockerhub_webhook_url {
  value = module.service.dockerhub_webhook_url
}