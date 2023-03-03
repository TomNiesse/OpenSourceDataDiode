#!/bin/bash
sudo systemctl disable --now osdd
tar -C /tmp -xzvf scripts/osdd_ingress.tar.gz --transform='s/.*\///' # or scripts/osdd_egress.tar.gz
sudo cp /tmp/osdd /home/osdd/
sudo chmod +x /home/osdd/osdd
sudo cp /tmp/Config.toml /home/osdd/
sudo chown osdd:osdd /home/osdd -R
sudo cp /tmp/osdd.service /etc/systemd/system
for docker_image in /tmp/*.tar
do
    sudo docker image rm `basename $docker_image .tar`
    sudo docker load -i $docker_image
done
sudo docker images
sudo systemctl daemon-reload
sudo systemctl start osdd
sudo systemctl enable osdd # optional
