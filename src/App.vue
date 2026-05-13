<template>
  <main class="app-shell">
    <section class="wizard" aria-labelledby="title">
      <aside class="wizard-sidebar">
        <div class="brand">
          <div class="brand-mark">G</div>
          <div>
            <p class="brand-name">GigaCoderAssist</p>
            <p class="brand-subtitle">{{ t.brandSubtitle }}</p>
          </div>
        </div>

        <ol class="steps" :aria-label="t.stepsLabel">
          <li
            v-for="item in steps"
            :key="item.id"
            class="step-item"
            :class="{ active: step === item.id, complete: stepIndex > item.index }"
          >
            <span class="step-dot">{{ item.index }}</span>
            <span>
              <strong>{{ item.title }}</strong>
              <small>{{ item.caption }}</small>
            </span>
          </li>
        </ol>

        <div class="sidebar-note">
          <strong>{{ t.writePolicyTitle }}</strong>
          <span>{{ t.writePolicyBody }}</span>
        </div>
      </aside>

      <section class="wizard-content">
        <header class="header">
          <div>
            <span class="eyebrow">{{ t.eyebrow }}</span>
            <h1 id="title">{{ currentStep.title }}</h1>
            <p>{{ currentStep.description }}</p>
          </div>
          <div class="header-tools">
            <div class="language-menu">
              <button
                class="language-trigger"
                type="button"
                :aria-label="t.language"
                :aria-expanded="languageMenuOpen"
                @click="languageMenuOpen = !languageMenuOpen"
              >
                <span>{{ currentLocaleOption.flag }}</span>
                <span>{{ currentLocaleOption.shortLabel }}</span>
                <span class="chevron" :class="{ open: languageMenuOpen }">⌃</span>
              </button>
              <div v-if="languageMenuOpen" class="language-popover" role="menu">
                <button
                  v-for="option in localeOptions"
                  :key="option.value"
                  class="language-option"
                  type="button"
                  role="menuitem"
                  @click="setLocale(option.value)"
                >
                  <span>{{ option.flag }}</span>
                  <span>{{ option.label }}</span>
                  <span v-if="locale === option.value" class="language-check">✓</span>
                </button>
              </div>
            </div>
            <span class="step-pill">{{ t.stepPrefix }} {{ stepIndex }} / 4</span>
          </div>
        </header>

        <div v-if="error" class="alert error">{{ error }}</div>
        <div v-if="notice" class="alert notice">{{ notice }}</div>

      <form v-if="step === 'login'" class="form surface" @submit.prevent="handleLogin">
          <div class="login-hero">
            <img class="login-icon" src="/app-icon.png" alt="GigaCoder" />
            <div>
              <strong>GigaCoderAssist</strong>
              <span>{{ t.loginHeroSubtitle }}</span>
            </div>
          </div>
          <label class="field">
            <span>{{ t.email }}</span>
            <input v-model.trim="email" type="email" autocomplete="username" required />
          </label>
          <label class="field">
            <span>{{ t.password }}</span>
            <input v-model="password" type="password" autocomplete="current-password" required />
          </label>
          <div class="form-footer">
            <p>{{ t.passwordHint }}</p>
            <button class="primary" type="submit" :disabled="loading">
              <span v-if="loading" class="spinner" aria-hidden="true"></span>
              {{ loading ? t.loggingIn : t.loginButton }}
            </button>
          </div>
        </form>

        <section v-else-if="step === 'select'" class="stack">
          <div class="surface">
            <div class="section-heading">
              <div>
                <h2>{{ t.selectKey }}</h2>
                <p>{{ userEmail }}</p>
              </div>
              <span class="count-badge">{{ t.availableKeys(keys.length) }}</span>
            </div>
            <div class="key-list">
              <button
                v-for="key in keys"
                :key="key.id"
                class="key-row"
                :class="{ selected: selectedKey?.id === key.id }"
                type="button"
                @click="selectedKey = key"
              >
                <span class="key-main">
                  <strong>{{ key.name }}</strong>
                  <small>{{ key.masked_key }}</small>
                </span>
                <span class="status">{{ key.status }}</span>
              </button>
            </div>
          </div>

          <div class="surface">
            <div class="section-heading">
              <div>
                <h2>{{ t.targets }}</h2>
                <p>{{ t.targetsHint }}</p>
              </div>
            </div>
            <div class="target-grid">
              <label class="target" :class="{ checked: configureClaude }">
                <input v-model="configureClaude" type="checkbox" />
                <span>
                  <strong>Claude Code</strong>
                  <small>~/.claude/settings.json</small>
                </span>
              </label>
              <label class="target" :class="{ checked: configureCodex }">
                <input v-model="configureCodex" type="checkbox" />
                <span>
                  <strong>Codex</strong>
                  <small>~/.codex/config.toml + auth.json</small>
                </span>
              </label>
            </div>
          </div>

          <div class="actions">
            <button type="button" @click="reset">{{ t.backToLogin }}</button>
            <button class="primary" type="button" :disabled="!canPreview || loading" @click="handlePreview">
              {{ loading ? t.loading : t.previewWrite }}
            </button>
          </div>
        </section>

        <section v-else-if="step === 'preview'" class="stack">
          <div class="surface">
            <div class="section-heading">
              <div>
                <h2>{{ t.filesToWrite }}</h2>
                <p>{{ t.filesToWriteHint }}</p>
              </div>
            </div>
            <ul class="path-list">
              <li v-if="preview?.claude_settings_path">
                <span>Claude Code</span>
                <code>{{ preview.claude_settings_path }}</code>
              </li>
              <li v-if="preview?.codex_config_path">
                <span>Codex config</span>
                <code>{{ preview.codex_config_path }}</code>
              </li>
              <li v-if="preview?.codex_auth_path">
                <span>Codex auth</span>
                <code>{{ preview.codex_auth_path }}</code>
              </li>
              <li v-if="preview?.codex_model_catalog_path">
                <span>Codex model catalog</span>
                <code>{{ preview.codex_model_catalog_path }}</code>
              </li>
            </ul>
          </div>
          <div class="actions">
            <button type="button" @click="step = 'select'">{{ t.back }}</button>
            <button class="primary" type="button" :disabled="loading" @click="handleApply">
              <span v-if="loading" class="spinner" aria-hidden="true"></span>
              {{ loading ? t.writing : t.confirmWrite }}
            </button>
          </div>
        </section>

        <section v-else class="stack">
          <div class="surface success-surface">
            <div class="success-mark">✓</div>
            <div>
              <h2>{{ t.doneTitle }}</h2>
              <p>{{ t.doneHint }}</p>
            </div>
          </div>
          <div class="result-list">
            <div v-for="file in resultFiles" :key="file.path" class="result-row">
              <span class="result-state">{{ file.created ? t.created : t.updated }}</span>
              <div>
                <code>{{ file.path }}</code>
                <small v-if="file.backup_path">{{ t.backup }}{{ file.backup_path }}</small>
              </div>
            </div>
          </div>
          <div class="actions">
            <button class="primary" type="button" @click="reset">{{ t.finish }}</button>
          </div>
        </section>
      </section>
    </section>
  </main>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type {
  ApiKeySummary,
  ApplyConfigResponse,
  ConfigPreview,
  FileWriteResult,
  LoginAndFetchKeysResponse
} from './types'

type Step = 'login' | 'select' | 'preview' | 'result'
type Locale = 'zh-Hant' | 'en'

const messages = {
  'zh-Hant': {
    brandSubtitle: '設定助手',
    stepsLabel: '設定步驟',
    writePolicyTitle: '寫入策略',
    writePolicyBody: '自動備份原設定，只更新 GigaCoder 相關欄位。',
    eyebrow: '本機環境設定',
    language: '語言',
    stepPrefix: '步驟',
    loginHeroSubtitle: '一鍵寫入 Codex 與 Claude Code 本機參數',
    email: 'GigaCoder 電子郵件',
    password: '密碼',
    passwordHint: '密碼只用於本次登入，不會寫入本機設定檔。',
    loggingIn: '登入中...',
    loginButton: '登入並讀取 Key',
    selectKey: '選擇 API Key',
    availableKeys: (count: number) => `${count} 個可用`,
    targets: '設定目標',
    targetsHint: '可以單獨設定，也可以同時設定兩個工具。',
    backToLogin: '返回登入',
    loading: '讀取中...',
    previewWrite: '預覽寫入',
    filesToWrite: '即將寫入這些檔案',
    filesToWriteHint: '已有檔案會先生成時間戳備份，然後合併更新設定。',
    back: '返回',
    writing: '寫入中...',
    confirmWrite: '確認寫入',
    doneTitle: '設定已寫入',
    doneHint: '重新開啟 Codex 或 Claude Code 後設定生效。',
    created: '已建立',
    updated: '已更新',
    backup: '備份：',
    finish: '完成',
    fallbackUsed: '未找到可直接使用的 Key，已取得預設 API Key。',
    noKeys: '目前帳號沒有可用於設定的 API Key',
    invalidCredentials: '使用者名稱或密碼不正確。',
    steps: {
      login: {
        title: '帳號登入',
        caption: '連接 GigaCoder',
        description: '使用 GigaCoder 帳號讀取可設定的 API Key。'
      },
      select: {
        title: '選擇設定',
        caption: 'Key 與工具',
        description: '選擇一個 API Key，並指定要寫入的開發工具。'
      },
      preview: {
        title: '寫入預覽',
        caption: '確認檔案路徑',
        description: '檢查將要修改的設定檔，寫入前會自動備份。'
      },
      result: {
        title: '完成',
        caption: '查看結果',
        description: '確認設定檔已建立或更新。'
      }
    }
  },
  en: {
    brandSubtitle: 'Config Assistant',
    stepsLabel: 'Setup steps',
    writePolicyTitle: 'Write policy',
    writePolicyBody: 'Back up existing files and update only GigaCoder-related fields.',
    eyebrow: 'Local environment setup',
    language: 'Language',
    stepPrefix: 'Step',
    loginHeroSubtitle: 'Write local Codex and Claude Code parameters in one flow',
    email: 'GigaCoder email',
    password: 'Password',
    passwordHint: 'Your password is only used for this login and is never written to local config files.',
    loggingIn: 'Signing in...',
    loginButton: 'Sign in and load keys',
    selectKey: 'Select API Key',
    availableKeys: (count: number) => `${count} available`,
    targets: 'Configuration targets',
    targetsHint: 'Configure one tool or both tools at the same time.',
    backToLogin: 'Back to sign in',
    loading: 'Loading...',
    previewWrite: 'Preview changes',
    filesToWrite: 'Files to be written',
    filesToWriteHint: 'Existing files will be timestamp-backed up before settings are merged.',
    back: 'Back',
    writing: 'Writing...',
    confirmWrite: 'Confirm write',
    doneTitle: 'Configuration written',
    doneHint: 'Restart Codex or Claude Code for the settings to take effect.',
    created: 'Created',
    updated: 'Updated',
    backup: 'Backup: ',
    finish: 'Finish',
    fallbackUsed: 'No directly usable key was found, so a default API Key was loaded.',
    noKeys: 'This account has no API Key available for configuration',
    invalidCredentials: 'Incorrect username or password.',
    steps: {
      login: {
        title: 'Account sign in',
        caption: 'Connect GigaCoder',
        description: 'Use your GigaCoder account to load configurable API keys.'
      },
      select: {
        title: 'Choose setup',
        caption: 'Key and tools',
        description: 'Select one API Key and the developer tools to configure.'
      },
      preview: {
        title: 'Write preview',
        caption: 'Confirm file paths',
        description: 'Review the config files before writing. Existing files are backed up first.'
      },
      result: {
        title: 'Complete',
        caption: 'Review result',
        description: 'Confirm that configuration files were created or updated.'
      }
    }
  }
} as const

const localeOptions: Array<{ value: Locale; flag: string; label: string; shortLabel: string }> = [
  { value: 'en', flag: '🇺🇸', label: 'English', shortLabel: 'EN' },
  { value: 'zh-Hant', flag: '🇨🇳', label: '中文繁體', shortLabel: 'ZH' }
]

const locale = ref<Locale>('en')
const languageMenuOpen = ref(false)
const t = computed(() => messages[locale.value])
const currentLocaleOption = computed(
  () => localeOptions.find((option) => option.value === locale.value) ?? localeOptions[0]
)

const steps = computed<Array<{ id: Step; index: number; title: string; caption: string; description: string }>>(() => [
  { id: 'login', index: 1, ...t.value.steps.login },
  { id: 'select', index: 2, ...t.value.steps.select },
  { id: 'preview', index: 3, ...t.value.steps.preview },
  { id: 'result', index: 4, ...t.value.steps.result }
])

const step = ref<Step>('login')
const email = ref('')
const password = ref('')
const userEmail = ref('')
const keys = ref<ApiKeySummary[]>([])
const selectedKey = ref<ApiKeySummary | null>(null)
const openaiModels = ref<string[]>([])
const configureClaude = ref(true)
const configureCodex = ref(true)
const preview = ref<ConfigPreview | null>(null)
const resultFiles = ref<FileWriteResult[]>([])
const loading = ref(false)
const error = ref('')
const notice = ref('')

const currentStep = computed(() => steps.value.find((item) => item.id === step.value) ?? steps.value[0])
const stepIndex = computed(() => currentStep.value.index)

const canPreview = computed(() => {
  return selectedKey.value !== null && (configureClaude.value || configureCodex.value)
})

function clearMessages() {
  error.value = ''
  notice.value = ''
}

function showError(err: unknown) {
  const message = err instanceof Error ? err.message : String(err)
  error.value = message === 'INVALID_CREDENTIALS' ? t.value.invalidCredentials : message
}

function setLocale(nextLocale: Locale) {
  locale.value = nextLocale
  languageMenuOpen.value = false
}

async function handleLogin() {
  clearMessages()
  loading.value = true
  try {
    const response = await invoke<LoginAndFetchKeysResponse>('login_and_fetch_keys_command', {
      email: email.value,
      password: password.value
    })
    password.value = ''
    userEmail.value = response.user_email
    keys.value = response.keys
    openaiModels.value = response.openai_models
    selectedKey.value = response.keys[0] ?? null
    notice.value = response.fallback_used ? t.value.fallbackUsed : ''
    if (response.keys.length === 0) {
      throw new Error(t.value.noKeys)
    }
    step.value = 'select'
  } catch (err) {
    showError(err)
  } finally {
    loading.value = false
  }
}

async function handlePreview() {
  if (!canPreview.value) return
  clearMessages()
  loading.value = true
  try {
    preview.value = await invoke<ConfigPreview>('preview_configuration_command', {
      configureClaude: configureClaude.value,
      configureCodex: configureCodex.value,
      openaiModels: openaiModels.value
    })
    step.value = 'preview'
  } catch (err) {
    showError(err)
  } finally {
    loading.value = false
  }
}

async function handleApply() {
  if (!selectedKey.value) return
  clearMessages()
  loading.value = true
  try {
    const response = await invoke<ApplyConfigResponse>('apply_configuration_command', {
      request: {
        api_key: selectedKey.value.raw_key,
        configure_claude: configureClaude.value,
        configure_codex: configureCodex.value,
        openai_models: openaiModels.value
      }
    })
    resultFiles.value = response.files
    step.value = 'result'
  } catch (err) {
    showError(err)
  } finally {
    loading.value = false
  }
}

function reset() {
  step.value = 'login'
  password.value = ''
  userEmail.value = ''
  keys.value = []
  openaiModels.value = []
  selectedKey.value = null
  preview.value = null
  resultFiles.value = []
  clearMessages()
}
</script>
