#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

if [ -e /dev/nvme1n1 ]; then
	if ! file -s /dev/nvme1n1 | grep -q filesystem; then
		mkfs.ext4 /dev/nvme1n1
	fi

	cat >> /etc/fstab <<-EOF
	/dev/nvme1n1  /data  ext4  defaults,noatime,nofail  0  2
	EOF

	mkdir /data
	mount /data
fi

mkdir -p /opt/libra

yum -y install awscli
aws s3 cp ${consensus_peers} /opt/libra/consensus_peers.config.toml
aws s3 cp ${network_peers} /opt/libra/network_peers.config.toml
aws s3 cp ${fullnode_peers} /opt/libra/fullnode_peers.config.toml
aws s3 cp ${genesis_blob} /opt/libra/genesis.blob

echo ECS_CLUSTER=${ecs_cluster} >> /etc/ecs/ecs.config
systemctl try-restart ecs --no-block

curl -o /tmp/node_exporter.rpm https://copr-be.cloud.fedoraproject.org/results/ibotty/prometheus-exporters/epel-7-x86_64/00935314-golang-github-prometheus-node_exporter/golang-github-prometheus-node_exporter-0.18.1-6.el7.x86_64.rpm
yum install -y /tmp/node_exporter.rpm
systemctl start node_exporter

cat > /etc/cron.d/metric_collector <<"EOF"
* * * * * root   docker container ls -q --filter label=com.amazonaws.ecs.container-name | xargs docker inspect --format='{{.State.StartedAt}}' | xargs date +"\%s" -d | xargs echo "ecs_start_time_seconds " > /var/lib/node_exporter/textfile_collector/ecs_stats.prom

* * * * * root	 docker container ls -q --filter label=com.amazonaws.ecs.container-name | xargs docker inspect --format='{{$tags := .Config.Labels}}build_info{revision="{{index $tags "org.label-schema.vcs-ref"}}", upstream="{{index $tags "vcs-upstream"}}"} 1' > /var/lib/node_exporter/textfile_collector/build_info.prom
EOF

cat > /etc/profile.d/libra_prompt.sh <<EOF
export PS1="[\u:validator@\h \w]$ "
EOF

yum -y install ngrep tcpdump perf gdb nmap-ncat strace htop sysstat tc
