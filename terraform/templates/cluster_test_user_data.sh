#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

cat > /libra_rsa <<'EOF'
${ssh_key}
EOF

chmod 600 /libra_rsa
chown ec2-user /libra_rsa

cat > /usr/local/bin/ct <<'EOF'
${ct}
EOF

chmod +x /usr/local/bin/ct

yum install git nano awscli -y
usermod -a -G docker ec2-user

cat > /etc/profile.d/libra_prompt.sh <<EOF
export PS1="[\u@cluster-test-runner \w]$ "
EOF
