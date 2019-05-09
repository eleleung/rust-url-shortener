variable "availability_zone" {
  default = "ap-southeast-2a"
}

variable "db_server_class" {}
variable "app_server_class" {}

variable "db_allocated_storage" {}

variable "db_final_snapshot_name" {}

variable "db_encrypted" {
  default = false
}

variable "db_name" {}
variable "db_username" {}
variable "db_password" {}

variable "key_name" {}

variable "env_name" {}

variable "auto_minor_version_upgrade" {
  default = "true"
}

variable "db_sg_description" {
  default = "Managed by Terraform"
}

variable "app_sg_description" {
  default = "Managed by Terraform"
}

provider "aws" {
  region = "ap-southeast-2"
}

output "ip" {
  value = "${aws_eip.ip.public_ip}"
}

output "db_host" {
  value = "${aws_db_instance.vromio_db.address}"
}

output "db_port" {
  value = "${aws_db_instance.vromio_db.port}"
}

output "key_name" {
  value = "${aws_instance.vromio_services.key_name}"
}

output "internal_dns" {
  value = "${aws_instance.vromio_services.private_dns}"
}

resource "aws_security_group" "db_sg" {
  description = "${var.db_sg_description}"

  ingress {
    from_port       = 0
    to_port         = 0
    protocol        = "-1"
    security_groups = ["${aws_security_group.app_sg.id}"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags {
    Name = "${var.env_name}-db-sg"
  }
}

resource "aws_security_group" "app_sg" {
  description = "${var.app_sg_description}"

  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port = 0
    to_port   = 0
    protocol  = "-1"
    self = true
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags {
    Name = "${var.env_name}-app-sg"
  }
}

resource "aws_instance" "vromio_services" {
  ami           = "ami-0b76c3b150c6b1423"
  instance_type = "${var.app_server_class}"
  key_name      = "${var.key_name}"

  tags {
    Name = "${var.env_name}-app"
  }

  vpc_security_group_ids = ["${aws_security_group.app_sg.id}"]
  availability_zone      = "${var.availability_zone}"

  root_block_device {
    volume_size = 50
  }
}

resource "aws_db_instance" "vromio_db" {
  identifier = "${var.env_name}-db"

  allocated_storage = "${var.db_allocated_storage}"
  storage_type      = "gp2"
  engine            = "postgres"
  instance_class    = "${var.db_server_class}"

  auto_minor_version_upgrade = "${var.auto_minor_version_upgrade}"

  name     = "${var.db_name}"
  username = "${var.db_username}"
  password = "${var.db_password}"

  multi_az          = false
  storage_encrypted = "${var.db_encrypted}"

  backup_window           = "07:00-08:00"
  backup_retention_period = 7

  availability_zone      = "${var.availability_zone}"
  vpc_security_group_ids = ["${aws_security_group.db_sg.id}"]

  final_snapshot_identifier = "${var.db_final_snapshot_name}"

  tags {
    Name = "${var.env_name}-db"
  }
}

resource "aws_eip" "ip" {
  instance = "${aws_instance.vromio_services.id}"

  tags {
    Name = "${var.env_name}-ip"
  }
}
