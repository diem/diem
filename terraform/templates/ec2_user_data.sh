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

	export log_path="/data/libra/${host_log_path}"
	export structlog_path="/data/libra/${host_structlog_path}"

	# non-persistent storage is managed by Docker under data-root
	if ! ${persistent} ; then
		cat >> /etc/fstab <<-EOF
		/dev/nvme1n1  /var/lib/docker/volumes  ext4  defaults,noatime,nofail  0  2
		EOF
		mkdir -p /var/lib/docker/volumes
		mount /var/lib/docker/volumes
		export log_path="/data/*/_data/${host_log_path}"
		export structlog_path="/data/*/_data/${host_structlog_path}"

		# give some helptul tips
		cat > /data/README <<-EOF
		In non-persistent mode -- data is not persisted between ECS updates. Showing Docker volumes instead. To find the currently mounted Docker volume:
		\$ docker container ls -q --filter label=com.amazonaws.ecs.container-name=validator | xargs docker inspect -f '{{ .Mounts }}' | awk '{print \$3}'
		EOF
	fi

fi

mkdir -p /opt/libra /vault

echo ECS_CLUSTER=${ecs_cluster} >> /etc/ecs/ecs.config
systemctl try-restart ecs --no-block

curl -o /tmp/node_exporter.rpm https://copr-be.cloud.fedoraproject.org/results/ibotty/prometheus-exporters/epel-7-x86_64/00935314-golang-github-prometheus-node_exporter/golang-github-prometheus-node_exporter-0.18.1-6.el7.x86_64.rpm
yum install -y /tmp/node_exporter.rpm
systemctl start node_exporter

cat > /etc/cron.d/metric_collector <<"EOF"
* * * * * root   docker container ls -q --filter label=vcs-upstream | xargs docker inspect --format='{{.State.StartedAt}}' | head -1 | xargs date +"\%s" -d | xargs echo "ecs_start_time_seconds " > /var/lib/node_exporter/textfile_collector/ecs_stats.prom

* * * * * root	 docker container ls -q --filter label=com.amazonaws.ecs.container-name | xargs docker inspect --format='{{$tags := .Config.Labels}}build_info{revision="{{index $tags "org.label-schema.vcs-ref"}}", upstream="{{index $tags "vcs-upstream"}}", container_name="{{index $tags "com.amazonaws.ecs.container-name"}}"} 1' > /var/lib/node_exporter/textfile_collector/build_info.prom
EOF

cat > /etc/profile.d/libra_prompt.sh <<EOF
export PS1="[\u:validator@\h \w]$ "
EOF

{% if enable_logrotate %}
cat > /etc/logrotate.d/libra <<EOF
hourly
${log_path} {
	maxsize 500M
	rotate 100
	compress
	delaycompress
	copytruncate
}

${structlog_path} {
	maxsize 500M
	rotate 100
	compress
	delaycompress
	copytruncate
}
EOF
sudo mv /etc/cron.daily/logrotate /etc/cron.hourly/
{% end %}

yum -y install ngrep tcpdump perf gdb nmap-ncat strace htop sysstat tc git

if [ ! -d /usr/local/etc/FlameGraph ] ; then
    git clone --depth 1 https://github.com/brendangregg/FlameGraph /usr/local/etc/FlameGraph
fi
#
