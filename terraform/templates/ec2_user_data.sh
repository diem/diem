#!/bin/bash

mkdir -p /opt/libra

yum -y install awscli
aws s3 cp ${trusted_peers} /opt/libra/trusted_peers.config.toml

echo ECS_CLUSTER=${ecs_cluster} >> /etc/ecs/ecs.config
systemctl try-restart ecs --no-block

curl -o /tmp/node_exporter.rpm https://copr-be.cloud.fedoraproject.org/results/ibotty/prometheus-exporters/epel-7-x86_64/00935314-golang-github-prometheus-node_exporter/golang-github-prometheus-node_exporter-0.18.1-6.el7.x86_64.rpm
yum install -y /tmp/node_exporter.rpm
systemctl start node_exporter

cat > /etc/cron.d/metric_collector <<"EOF"
* * * * * root   docker container ls -q --filter label=com.amazonaws.ecs.container-name | xargs docker inspect --format='{{.State.StartedAt}}' | xargs date +"\%s" -d | xargs echo "ecs_start_time_seconds " > /var/lib/node_exporter/textfile_collector/ecs_stats.prom

* * * * * root	 docker container ls -q --filter label=com.amazonaws.ecs.container-name | xargs docker inspect --format='{{$tags := .Config.Labels}}build_info{revision="{{index $tags "org.label-schema.vcs-ref"}}", upstream="{{index $tags "vcs-upstream"}}"} 1' > /var/lib/node_exporter/textfile_collector/build_info.prom
EOF

yum -y install ngrep tcpdump perf gdb nmap-ncat strace htop sysstat
