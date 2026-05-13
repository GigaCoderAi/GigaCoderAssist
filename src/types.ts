export interface ApiKeySummary {
  id: string
  name: string
  masked_key: string
  raw_key: string
  status: string
  expires_at: string | null
}

export interface LoginAndFetchKeysResponse {
  user_email: string
  keys: ApiKeySummary[]
  platforms: PlatformSummary[]
  models: string[]
  openai_models: string[]
  fallback_used: boolean
}

export interface PlatformSummary {
  code: string
  name: string
}

export interface ConfigPreview {
  claude_settings_path?: string
  codex_config_path?: string
  codex_auth_path?: string
  codex_model_catalog_path?: string
}

export interface ApplyConfigRequest {
  api_key: string
  configure_claude: boolean
  configure_codex: boolean
  openai_models: string[]
}

export interface FileWriteResult {
  path: string
  backup_path?: string
  created: boolean
}

export interface ApplyConfigResponse {
  files: FileWriteResult[]
}
