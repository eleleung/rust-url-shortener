terraform {
  backend "s3" {
    bucket = "vromio-tfstate-backend"
    key    = "prod"
    region = "ap-southeast-2"
  }
}

variable "appkey" {
  default = "vromio"
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

  app_sg_description = "Production security group"
  db_sg_description  = "production rules for mysql"

  key_name = "${var.appkey}"

  env_name = "vromio-prod"
}

output "ip" { value = "${module.vromio.ip}" }

output "db_host" { value = "${module.vromio.db_host}" }
output "db_port" { value = "${module.vromio.db_port}" }

output "key_name" { value = "${var.appkey}" }

output "internal_dns" {
  value = "${module.vromio.internal_dns}"
}