{{- define "backup.backupEnvironment" -}}
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
