{{/* vim: set filetype=mustache: */}}
{{/*
Expand the name of the chart.
*/}}
{{- define "diem-fullnode.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "diem-fullnode.fullname" -}}
{{- if .Values.fullnameOverride -}}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- $name := default .Chart.Name .Values.nameOverride -}}
{{- if contains $name .Release.Name -}}
{{- .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- end -}}
{{- end -}}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "diem-fullnode.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
Common labels
*/}}
{{- define "diem-fullnode.labels" -}}
helm.sh/chart: {{ include "diem-fullnode.chart" . }}
{{ include "diem-fullnode.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{/*
Selector labels
*/}}
{{- define "diem-fullnode.selectorLabels" -}}
app.kubernetes.io/part-of: {{ include "diem-fullnode.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{/*
Create the name of the service account to use
*/}}
{{- define "diem-fullnode.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
    {{ default (include "diem-fullnode.fullname" .) .Values.serviceAccount.name }}
{{- else -}}
    {{ default "default" .Values.serviceAccount.name }}
{{- end -}}
{{- end -}}

{{- define "diem-fullnode.backupEnvironment" -}}
# awscli writes to ~/.aws/cli/cache/
# gsutil writes to ~/.gsutil/
# azcopy writes to ~/.azcopy/
- name: HOME
  value: /tmp
{{- if hasPrefix "s3" (toString .config.location) }}
- name: BUCKET
  value: {{ .config.s3.bucket }}
{{- end }}
{{- if hasPrefix "gcs" (toString .config.location) }}
- name: BUCKET
  value: {{ .config.gcs.bucket }}
{{- end }}
{{- if hasPrefix "azure" (toString .config.location) }}
- name: ACCOUNT
  value: {{ .config.azure.account }}
- name: CONTAINER
  value: {{ .config.azure.container }}
- name: SAS
  value: {{ .config.azure.sas }}
{{- end }}
{{- if hasPrefix "scw_s3" (toString .config.location) }}
- name: AWS_ACCESS_KEY_ID
  value: {{ .config.scw_s3.access_key }}
- name: AWS_SECRET_ACCESS_KEY
  value: {{ .config.scw_s3.secret_key }}
- name: AWS_DEFAULT_REGION
  value: {{ .config.scw_s3.region }}
- name: BUCKET
  value: {{ .config.scw_s3.bucket }}
- name: ENDPOINT_URL
  value: {{ .config.scw_s3.endpoint_url }}
{{- end }}
- name: SUB_DIR
  value: e{{ .era }}
{{- end -}}

{{- define "backup.fullname" -}}
{{ include "diem-fullnode.fullname" . }}
{{- end -}}

{{- define "backup.labels" -}}
{{ include "diem-fullnode.labels" . }}
{{- end -}}

{{- define "backup.selectorLabels" -}}
{{ include "diem-fullnode.selectorLabels" . }}
{{- end -}}

{{- define "backup.serviceAccount" -}}
{{ include "diem-fullnode.serviceAccountName" . }}
{{- end -}}

{{- define "backup.backupService" -}}
{{ include "diem-fullnode.fullname" . }}:6186
{{- end -}}

{{- define "backup.persistentVolumeClaim" -}}
{{ include "diem-fullnode.fullname" . }}-e{{ .Values.chain.era }}
{{- end -}}

{{- define "backup.pushGateway" -}}
{{- if .Values.backup.pushGateway -}}
{{ .Values.backup.pushGateway }}
{{- end -}}
{{- end -}}

{{- define "backup.loggingAddress" -}}
{{ .Values.logging.address }}
{{- end -}}
