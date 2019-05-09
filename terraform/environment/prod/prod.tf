terraform {
  backend "s3" {
    bucket = "vromio-tfstate"
    key    = "prod"
    region = "ap-southeast-2"
  }
}

variable "appkey" {
  default = "vromio"
}

variable "env_name" {
  default = "vromio-prod"
}

variable "db_password" {}

module "vromio" {
  source = "../../modules/vromio"

  app_server_class = "t3.small"

  db_server_class        = "db.t3.small"
  db_allocated_storage   = 1500
  db_final_snapshot_name = "vromio-prod-db-snapshot"

  db_name     = ""
  db_username = "root"
  db_password = "${var.db_password}"

  auto_minor_version_upgrade = "false"

  key_name = "${var.appkey}"

  env_name = "${var.env_name}"
}

output "ip" { value = "${module.vromio.ip}" }

output "db_host" { value = "${module.vromio.db_host}" }
output "db_port" { value = "${module.vromio.db_port}" }
output "db_password" { value = "${var.db_password}" }

output "key_name" { value = "${var.appkey}" }

output "env_name" { value = "${var.env_name}" }

output "domains" { value = ["vrom.io"] }

output "internal_dns" {
  value = "${module.vromio.internal_dns}"
}