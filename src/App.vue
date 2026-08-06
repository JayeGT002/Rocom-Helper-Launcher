<template>
  <div class="app">
    <!-- Settings gear -->
    <button class="gear" @click="showSettings = true" title="设置">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="3"/>
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
      </svg>
    </button>

    <!-- Version tag -->
    <div class="version-tag" v-if="status.latest || status.local || checking || githubOk !== null">
      <span v-if="githubOk === true" class="gh-ok" title="GitHub 连通正常">●</span>
      <span v-else-if="githubOk === false" class="gh-fail" title="GitHub 连接失败">●</span>
      <span v-if="status.latest" class="ver latest">最新 v{{ status.latest }}</span>
      <template v-if="status.latest && status.local">
        <span class="ver-arrow">→</span>
        <span class="ver local">本地 v{{ status.local }}</span>
      </template>
      <span v-else-if="status.latest && !status.local" class="ver none">未安装</span>
      <span v-else-if="checking" class="ver checking">检测中...</span>
    </div>

    <!-- Main content -->
    <div class="main">
      <!-- Left panel: buttons -->
      <div class="left-panel">
        <button class="btn-primary" @click="launchAll" :disabled="busy">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
          <span>启动游戏 / WeGame</span>
          <small>先启 Rocom Helper → 再启游戏</small>
        </button>

        <button class="btn-secondary" @click="launchRocomOnly" :disabled="busy">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5,3 19,12 5,21" fill="currentColor"/></svg>
          <span>仅启动 Rocom Helper</span>
        </button>

        <transition name="fade">
          <button v-if="needUpdate" class="btn-update" @click="doUpdate" :disabled="busy">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7,10 12,15 17,10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
            更新到 v{{ status.latest }}
          </button>
        </transition>

        <button class="btn-redownload" @click="redownload" :disabled="busy">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M23 4v6h-6"/><path d="M1 20v-6h6"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10"/><path d="M20.49 15a9 9 0 0 1-14.85 3.36L1 14"/></svg>
          重新下载
        </button>

        <div class="busy-bar" v-if="busy">
          <div class="spinner"></div>
          <span>{{ busyText }}</span>
        </div>
      </div>

      <!-- Right panel: logs -->
      <div class="right-panel">
        <div class="log-header">
          <span>运行日志</span>
          <button class="log-clear" @click="logs = []" v-if="logs.length">清空</button>
        </div>
        <div class="log-box" ref="logBox">
          <div v-for="(l, i) in logs" :key="i" :class="'log-line ' + l.level">
            <span class="log-time">{{ l.time }}</span>
            <span class="log-msg">{{ l.msg }}</span>
          </div>
          <div v-if="logs.length === 0" class="log-empty">等待操作...</div>
        </div>
      </div>
    </div>

    <!-- Settings modal -->
    <transition name="modal">
      <div class="modal-overlay" v-if="showSettings" @click.self="showSettings = false">
        <div class="modal">
          <div class="modal-header">
            <h2>设置</h2>
            <button class="modal-close" @click="showSettings = false">&times;</button>
          </div>
          <div class="modal-body">
            <label class="field">
              <span class="field-label">游戏路径</span>
              <div class="path-row">
                <input v-model="settings.game_path" placeholder="留空则跳过启动游戏" />
                <button @click="browse('game_path')">浏览</button>
              </div>
            </label>
            <label class="field">
              <span class="field-label">WeGame 路径</span>
              <div class="path-row">
                <input v-model="settings.wegame_path" placeholder="留空则跳过启动 WeGame" />
                <button @click="browse('wegame_path')">浏览</button>
              </div>
            </label>
            <label class="field">
              <span class="field-label">Rocom Helper 存放路径</span>
              <div class="path-row">
                <input v-model="settings.store_path" placeholder="下载文件存放位置" />
                <button @click="browse('store_path')">浏览</button>
              </div>
            </label>
            <label class="toggle-field">
              <span class="field-label">启动前询问是否更新版本</span>
              <button class="toggle" :class="{ on: settings.ask_before_update }" @click="settings.ask_before_update = !settings.ask_before_update">
                <span class="toggle-knob"></span>
              </button>
            </label>
          </div>
          <div class="modal-footer">
            <button class="btn-save" @click="saveSettings">保存设置</button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted, nextTick, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open, ask } from "@tauri-apps/plugin-dialog";

const checking = ref(false);
const busy = ref(false);
const busyText = ref("");
const showSettings = ref(false);
const logBox = ref(null);
const logs = ref([]);
const githubOk = ref(null);

const status = reactive({
  latest: null,
  local: null,
});

const settings = reactive({
  game_path: "",
  wegame_path: "",
  store_path: "",
  ask_before_update: true,
});

const needUpdate = computed(() => {
  return status.latest && status.latest !== status.local;
});

function addLog(level, msg) {
  const now = new Date();
  const time = `${String(now.getHours()).padStart(2, "0")}:${String(now.getMinutes()).padStart(2, "0")}:${String(now.getSeconds()).padStart(2, "0")}`;
  logs.value.push({ level, msg, time });
  nextTick(() => {
    if (logBox.value) logBox.value.scrollTop = logBox.value.scrollHeight;
  });
}

async function checkVersion() {
  checking.value = true;
  try {
    const res = await invoke("check_version");
    status.latest = res.latest;
    status.local = res.local;
    if (res.local) {
      addLog("info", `本地版本: v${res.local}`);
    }
    if (res.latest) {
      addLog("success", `最新版本: v${res.latest}`);
    }
    if (res.latest && !res.local) {
      addLog("warning", "本地未安装，需要下载");
    } else if (res.latest && res.latest !== res.local) {
      addLog("warning", `有新版本可用: v${res.latest}`);
    } else if (res.latest && res.latest === res.local) {
      addLog("success", "已是最新版本");
    }
  } catch (e) {
    addLog("error", `版本检测失败: ${e}`);
  } finally {
    checking.value = false;
  }
}

async function loadSettings() {
  try {
    const s = await invoke("get_settings");
    Object.assign(settings, s);
  } catch (e) {
    addLog("error", `加载设置失败: ${e}`);
  }
}

async function saveSettings() {
  try {
    await invoke("save_settings", { settings: { ...settings } });
    addLog("success", "设置已保存");
    showSettings.value = false;
  } catch (e) {
    addLog("error", `保存设置失败: ${e}`);
  }
}

async function browse(field) {
  try {
    const selected = await open({
      directory: field === "store_path",
      multiple: false,
    });
    if (selected) {
      settings[field] = selected;
    }
  } catch (e) {
    addLog("error", `选择路径失败: ${e}`);
  }
}

async function launchRocomOnly() {
  if (busy.value) return;
  busy.value = true;
  busyText.value = "正在启动 Rocom Helper...";
  addLog("info", "启动 Rocom Helper...");
  try {
    await invoke("launch_rocom");
    addLog("success", "Rocom Helper 已启动");
  } catch (e) {
    addLog("error", `启动失败: ${e}`);
  } finally {
    busy.value = false;
  }
}

async function launchAll() {
  if (busy.value) return;

  if (needUpdate.value && settings.ask_before_update) {
    addLog("warning", "检测到新版本，请先更新或关闭询问后直接启动");
    return;
  }

  if (!status.local && status.latest) {
    addLog("warning", "本地未安装 Rocom Helper，正在下载...");
    busy.value = true;
    busyText.value = "正在下载 Rocom Helper...";
    try {
      await invoke("download_helper");
      await invoke("check_version");
      addLog("success", "下载完成");
    } catch (e) {
      addLog("error", `下载失败: ${e}`);
      busy.value = false;
      return;
    }
    busy.value = false;
  }

  busy.value = true;
  busyText.value = "正在启动...";
  addLog("info", "启动 Rocom Helper...");
  try {
    await invoke("launch_rocom");
    addLog("success", "Rocom Helper 已启动");

    if (settings.game_path) {
      addLog("info", "启动游戏...");
      await invoke("launch_game", { gameType: "game" });
      addLog("success", "游戏已启动");
    }
    if (settings.wegame_path) {
      addLog("info", "启动 WeGame...");
      await invoke("launch_game", { gameType: "wegame" });
      addLog("success", "WeGame 已启动");
    }
  } catch (e) {
    addLog("error", `启动失败: ${e}`);
  } finally {
    busy.value = false;
  }
}

async function doUpdate() {
  if (busy.value) return;
  busy.value = true;
  busyText.value = "正在更新 Rocom Helper...";
  addLog("info", "开始更新...");
  try {
    await invoke("download_helper");
    await invoke("check_version");
    addLog("success", "更新完成");
  } catch (e) {
    addLog("error", `更新失败: ${e}`);
  } finally {
    busy.value = false;
  }
}

async function testGithub() {
  try {
    const ok = await invoke("test_github_connectivity");
    githubOk.value = ok;
    return ok;
  } catch (e) {
    githubOk.value = false;
    addLog("error", `GitHub 连通性测试失败: ${e}`);
    return false;
  }
}

async function redownload() {
  if (busy.value) return;

  if (status.local) {
    const confirmed = await ask(
      `本地已存在 Rocom Helper (v${status.local})，是否删除并重新下载？`,
      { title: "确认重新下载", kind: "warning" }
    );
    if (!confirmed) {
      addLog("info", "已取消重新下载");
      return;
    }
    addLog("info", "正在删除本地文件...");
    try {
      await invoke("delete_helper");
    } catch (e) {
      addLog("error", `删除失败: ${e}`);
      return;
    }
  } else {
    addLog("info", "本地未安装，开始下载...");
  }

  busy.value = true;
  busyText.value = "正在下载 Rocom Helper...";
  try {
    await invoke("download_helper");
    await invoke("check_version");
    addLog("success", "下载完成");
  } catch (e) {
    addLog("error", `下载失败: ${e}`);
  } finally {
    busy.value = false;
  }
}

onMounted(async () => {
  await loadSettings();
  addLog("info", "应用已启动");

  listen("backend-log", (event) => {
    const { level, msg } = event.payload;
    addLog(level, msg);
  });

  const ok = await testGithub();
  if (ok) {
    await checkVersion();
  } else {
    addLog("error", "GitHub 不可达，版本检测已跳过。请检查网络连接。");
  }
});
</script>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg);
  position: relative;
  padding: 16px 20px 16px 20px;
  gap: 12px;
}

.gear {
  position: absolute;
  top: 14px;
  right: 16px;
  background: transparent;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
  padding: 6px;
  border-radius: 8px;
  transition: all 0.2s;
  z-index: 10;
}
.gear:hover {
  color: var(--text);
  background: var(--surface-hover);
}

.version-tag {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  padding: 0 38px;
}
.ver {
  padding: 2px 10px;
  border-radius: 20px;
  font-weight: 500;
}
.ver.latest {
  background: rgba(79, 140, 255, 0.15);
  color: var(--primary);
  border: 1px solid rgba(79, 140, 255, 0.3);
}
.ver.local {
  background: rgba(0, 212, 170, 0.15);
  color: var(--accent);
  border: 1px solid rgba(0, 212, 170, 0.3);
}
.ver.none {
  background: rgba(255, 165, 2, 0.15);
  color: var(--warning);
  border: 1px solid rgba(255, 165, 2, 0.3);
}
.ver.checking {
  background: var(--surface);
  color: var(--text-dim);
  border: 1px solid var(--border);
}
.ver-arrow {
  color: var(--text-faint);
  font-size: 11px;
}
.gh-ok {
  color: var(--accent);
  font-size: 10px;
  margin-right: 2px;
}
.gh-fail {
  color: var(--danger);
  font-size: 10px;
  margin-right: 2px;
}

.main {
  flex: 1;
  display: flex;
  gap: 16px;
  min-height: 0;
}

/* Left panel */
.left-panel {
  flex: 0 0 300px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  justify-content: center;
}

.btn-primary {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 24px 20px;
  background: linear-gradient(135deg, var(--primary), #3a78f0);
  border: none;
  border-radius: var(--radius);
  color: white;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 4px 16px var(--primary-glow);
}
.btn-primary:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 6px 24px var(--primary-glow);
}
.btn-primary:active:not(:disabled) {
  transform: translateY(0);
}
.btn-primary span {
  font-size: 16px;
  font-weight: 600;
}
.btn-primary small {
  font-size: 11px;
  opacity: 0.7;
  font-weight: 400;
}

.btn-secondary {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 14px 20px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  color: var(--text);
  cursor: pointer;
  transition: all 0.2s;
  font-size: 14px;
  font-weight: 500;
}
.btn-secondary:hover:not(:disabled) {
  background: var(--surface-hover);
  border-color: var(--text-faint);
}

.btn-update {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px 16px;
  background: rgba(0, 212, 170, 0.1);
  border: 1px solid rgba(0, 212, 170, 0.4);
  border-radius: var(--radius);
  color: var(--accent);
  cursor: pointer;
  transition: all 0.2s;
  font-size: 13px;
  font-weight: 500;
}
.btn-update:hover:not(:disabled) {
  background: rgba(0, 212, 170, 0.2);
}

.btn-redownload {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px 16px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  color: var(--text-dim);
  cursor: pointer;
  transition: all 0.2s;
  font-size: 13px;
  font-weight: 500;
}
.btn-redownload:hover:not(:disabled) {
  background: var(--surface-hover);
  color: var(--text);
  border-color: var(--text-faint);
}
.btn-redownload:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-primary:disabled,
.btn-secondary:disabled,
.btn-update:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.busy-bar {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: var(--text-dim);
  font-size: 12px;
}
.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--border);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Right panel */
.right-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
  min-width: 0;
}

.log-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border);
  font-size: 13px;
  font-weight: 600;
  color: var(--text-dim);
}
.log-clear {
  background: transparent;
  border: none;
  color: var(--text-faint);
  cursor: pointer;
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  transition: all 0.2s;
}
.log-clear:hover {
  color: var(--text);
  background: var(--surface-hover);
}

.log-box {
  flex: 1;
  overflow-y: auto;
  padding: 8px 14px;
  font-family: "Cascadia Code", "Consolas", "Courier New", monospace;
  font-size: 12px;
  line-height: 1.8;
}
.log-line {
  display: flex;
  gap: 8px;
  white-space: pre-wrap;
  word-break: break-all;
}
.log-time {
  color: var(--text-faint);
  flex-shrink: 0;
}
.log-msg { color: var(--text); }
.log-line.info .log-msg { color: var(--text-dim); }
.log-line.success .log-msg { color: var(--accent); }
.log-line.warning .log-msg { color: var(--warning); }
.log-line.error .log-msg { color: var(--danger); }
.log-empty {
  color: var(--text-faint);
  text-align: center;
  padding: 20px;
}

/* Modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  backdrop-filter: blur(4px);
}
.modal {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 16px;
  width: 460px;
  max-width: 90vw;
  box-shadow: var(--shadow);
  overflow: hidden;
}
.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}
.modal-header h2 {
  font-size: 16px;
  font-weight: 600;
}
.modal-close {
  background: transparent;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
  font-size: 22px;
  line-height: 1;
  padding: 0 4px;
  transition: color 0.2s;
}
.modal-close:hover { color: var(--text); }

.modal-body {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.field-label {
  font-size: 12px;
  color: var(--text-dim);
  font-weight: 500;
}
.path-row {
  display: flex;
  gap: 8px;
}
.path-row input {
  flex: 1;
  padding: 8px 12px;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text);
  font-size: 13px;
  outline: none;
  transition: border-color 0.2s;
  user-select: text;
}
.path-row input:focus {
  border-color: var(--primary);
}
.path-row button {
  padding: 8px 14px;
  background: var(--surface-hover);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text);
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s;
  white-space: nowrap;
}
.path-row button:hover {
  background: var(--border);
}

.toggle-field {
  display: flex;
  align-items: center;
  justify-content: space-between;
  cursor: pointer;
}
.toggle {
  width: 40px;
  height: 22px;
  border-radius: 11px;
  background: var(--border);
  border: none;
  cursor: pointer;
  position: relative;
  transition: background 0.2s;
  padding: 0;
}
.toggle.on {
  background: var(--primary);
}
.toggle-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: white;
  transition: transform 0.2s;
}
.toggle.on .toggle-knob {
  transform: translateX(18px);
}

.modal-footer {
  padding: 16px 20px;
  border-top: 1px solid var(--border);
  display: flex;
  justify-content: flex-end;
}
.btn-save {
  padding: 10px 28px;
  background: var(--primary);
  border: none;
  border-radius: 8px;
  color: white;
  cursor: pointer;
  font-size: 13px;
  font-weight: 600;
  transition: all 0.2s;
}
.btn-save:hover {
  background: var(--primary-hover);
}

/* Transitions */
.fade-enter-active, .fade-leave-active {
  transition: all 0.2s;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
  transform: translateY(8px);
}
.modal-enter-active, .modal-leave-active {
  transition: opacity 0.2s;
}
.modal-enter-from, .modal-leave-to {
  opacity: 0;
}
.modal-enter-active .modal, .modal-leave-active .modal {
  transition: transform 0.2s;
}
.modal-enter-from .modal, .modal-leave-to .modal {
  transform: scale(0.95);
}
</style>
