<template>
  <v-app @wheel.prevent @touchmove.prevent @scroll.prevent>
    <v-main>
      <v-container class="mx-auto">
        <div class="profile-row mt-3">
          <ProfileView class="profile-trigger" ref="profileView" @profileChanged="profileSelected" />
          <div class="header-actions">
            <v-btn size="small" variant="text" color="primary" prepend-icon="mdi-sd" @click="openGenerateSdCardDialog">
              Generate SDCard Folder
            </v-btn>
            <v-btn size="small" variant="text" color="primary" prepend-icon="mdi-file-document-outline" @click="openLogsDialog">
              Logs
            </v-btn>
            <v-btn size="small" variant="text" color="primary" prepend-icon="mdi-information-outline" @click="showAbout = true">
              About
            </v-btn>
          </div>
        </div>

        <OptionCard class="mt-3 opt-card" :cardTitle="'SSBU Settings'"
          :cardSubtitle="'Optimize emulator graphics and CPU settings for SSBU'" :cardDisplayIcon="'mdi-cog'"
          :isOptimized="user_status.settings_optimized && selected_profile != null"
          @updated="(s, o) => { optUpdated('Settings', s, o) }" />
        <OptionCard class="mt-3 opt-card" :cardTitle="'SSBU Mods'"
          :cardSubtitle="'Add useful mods for training and online play'" :cardDisplayIcon="'mdi-folder-wrench'"
          :isOptimized="user_status.mods_optimized && selected_profile != null"
          :cleanupOptions="[{ id: 'CleanAtmosphereAppDir', label: 'Clean Atmosphere App Folder' }, { id: 'CleanUltimateDir', label: 'Clean SSBU Mod Folder' }]"
          :modOptions="manifest_mods"
          @updated="(s, o) => { optUpdated('Mods', s, o) }" />
        <OptionCard class="mt-3 opt-card" :cardTitle="'Save Data'"
          :cardSubtitle="'Overwrite SSBU save with a 100% save for competitive play'"
          :cardDisplayIcon="'mdi-content-save-all'" :isOptimized="user_status.save_optimized && selected_profile != null"
          @updated="(s, o) => { optUpdated('Save', s, o) }" />

        <v-card-item class="justify-center" style="padding-top: 25px;">
          <v-tooltip location="right" :disabled="selected_profile != null && isAnyOptsEnabled && !isOptimizing">
            <template v-slot:activator="{ props }">
              <div v-bind="props" class="d-inline-block">
                <v-btn color="primary" :disabled="selected_profile == null || !isAnyOptsEnabled || isOptimizing" @click="optimizeSelected">Optimize
                  Selected</v-btn>
              </div>
            </template>
            <span v-if="selected_profile == null">Incorrect Emulator Setup</span>
            <span v-if="!isAnyOptsEnabled">No Option Selected</span>
            <span v-if="isOptimizing">Optimization in progress</span>
          </v-tooltip>
        </v-card-item>
      </v-container>
      <v-overlay :model-value="isOptimizing" class="align-center justify-center" persistent>
        <v-card class="px-6 py-5 text-center">
          <v-progress-circular indeterminate color="primary" size="46" width="4" />
          <div class="text-subtitle-1 mt-3">{{ loadingTitle }}</div>
          <div v-if="backendOptimizationMessage" class="text-caption mt-1">{{ backendOptimizationMessage }}</div>
          <div v-else-if="currentOptimization" class="text-caption mt-1">Current: {{ currentOptimization }}</div>
          <v-progress-linear
            v-if="optimizationProgress !== null"
            class="mt-3"
            color="primary"
            :model-value="optimizationProgress"
            height="8"
            rounded
          />
        </v-card>
      </v-overlay>
      <v-dialog v-model="showAbout" max-width="360">
        <v-card title="About">
          <v-card-text>
            <div class="text-body-2">SSBU Emulator Optimizer</div>
            <div class="text-caption mt-1">Version {{ appVersion || 'unknown' }}</div>
            <div class="text-caption mt-1">Build {{ appBuildUid || 'unknown' }}</div>
          </v-card-text>
          <v-card-actions>
            <v-spacer />
            <v-btn variant="text" color="primary" @click="showAbout = false">Close</v-btn>
          </v-card-actions>
        </v-card>
      </v-dialog>
      <v-dialog v-model="showLogs" max-width="880">
        <v-card title="Logs">
          <v-card-text>
            <v-sheet class="logs-content" border rounded>
              <div v-if="logsLoading" class="logs-loading">
                <v-progress-circular indeterminate color="primary" size="28" width="3" />
              </div>
              <pre v-else ref="logsPre" class="logs-pre">{{ logsContent || 'No logs available.' }}</pre>
            </v-sheet>
          </v-card-text>
          <v-card-actions>
            <v-btn variant="text" color="primary" @click="refreshLogs">Refresh</v-btn>
            <v-btn variant="text" color="primary" @click="openLogsFolder">Open Logs Folder</v-btn>
            <v-spacer />
            <v-btn variant="text" color="primary" @click="showLogs = false">Close</v-btn>
          </v-card-actions>
        </v-card>
      </v-dialog>
      <v-dialog v-model="showGenerateSdCard" max-width="760">
        <v-card title="Generate SDCard Folder">
          <v-card-subtitle>Select which mods should be included in the generated root folder.</v-card-subtitle>
          <v-card-text>
            <ModSelectionChecklist
              section-title="Mods To Include"
              :mods="manifest_mods"
              :selected-mod-ids="generateSelectedModIds"
              :selected-mod-sources="generateSelectedModSources"
              @update:selected-mod-ids="onGenerateSelectedModIdsUpdated"
              @update:selected-mod-sources="onGenerateSelectedModSourcesUpdated"
            />
          </v-card-text>
          <v-card-actions>
            <v-spacer />
            <v-btn variant="text" color="primary" :disabled="isOptimizing" @click="showGenerateSdCard = false">Cancel</v-btn>
            <v-btn variant="flat" color="primary" :disabled="isOptimizing || generateSelectedModIds.length === 0" @click="generateSdCardFolder">Generate</v-btn>
          </v-card-actions>
        </v-card>
      </v-dialog>
      <div>
        <v-snackbar v-for="(s, i) in snackbars" v-model="s.show" :key="i" :color="s.color" transition="fade-transition"
          :timeout="(s.timeout - 500)" :style="{ 'margin-bottom': calcSnackbarMargin(i) }">
          {{ s.text }}
        </v-snackbar>
      </div>
    </v-main>
  </v-app>
</template>

<script>
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { info, error } from "@tauri-apps/plugin-log";
import { ask } from '@tauri-apps/plugin-dialog';
import { check } from '@tauri-apps/plugin-updater';
import { nextTick, ref } from 'vue';


export default {
  data() {
    return {
      config: null,
      selected_profile: null,
      user_status: {
        settings_optimized: false,
        mods_optimized: false,
        save_optimized: false,
      },
      selected_opts: {
        "Settings": {
          enabled: true,
          options: [],
        },
        "Mods": {
          enabled: true,
          options: [],
        },
        "Save": {
          enabled: true,
          options: [],
        },
      },
      manifest_mods: [],
      snackbars: [],
      isOptimizing: false,
      currentOptimization: null,
      showAbout: false,
      appVersion: null,
      appBuildUid: null,
      backendOptimizationMessage: null,
      optimizationProgress: null,
      unlistenOptimizationStatus: null,
      showLogs: false,
      logsLoading: false,
      logsContent: '',
      showGenerateSdCard: false,
      generateSelectedModIds: [],
      generateSelectedModSources: {},
    };
  },
  setup() {
    const profileView = ref(null);
    return {
      profileView
    }
  },
  mounted() {
    invoke('query_config').then((c) => {
      this.config = c;
      this.$refs.profileView.init(this.config);
      info('Config Loaded: ' + JSON.stringify(this.config, null, 2));
    }).catch((err) => {
      error(err);
    });

    this.checkForUpdates();
    this.loadManifestMods();
    this.loadAppVersion();
    this.loadAppBuildUid();
    this.bindOptimizationStatusEvents();
  },
  beforeUnmount() {
    if (this.unlistenOptimizationStatus) {
      this.unlistenOptimizationStatus();
      this.unlistenOptimizationStatus = null;
    }
  },
  computed: {
    isAnyOptsEnabled() {
      return (
        this.selected_opts.Settings.enabled ||
        this.selected_opts.Mods.enabled ||
        this.selected_opts.Save.enabled
      )
    },
    loadingTitle() {
      return this.currentOptimization === 'GenerateSdCard'
        ? 'Generating SDCard folder...'
        : 'Applying optimizations...';
    }
  },
  methods: {
    async checkForUpdates() {
      try {
        const update = await check();
        if (!update) {
          info('No update available.');
          return;
        }

        const shouldInstall = await ask(
          `Version ${update.version} is available (current ${update.currentVersion}). Download and install now?`,
          {
            title: 'Update Available',
            kind: 'info',
            okLabel: 'Install',
            cancelLabel: 'Later',
          }
        );

        if (!shouldInstall) {
          info(`Update ${update.version} deferred by user.`);
          return;
        }

        this.showSnackbar(`Downloading update ${update.version}...`, 4000, 'blue');
        await update.downloadAndInstall();
        this.showSnackbar('Update installed. Restarting app...', 3000, 'green');
        await invoke('restart_app');
      } catch (err) {
        error(`Auto updater error: ${err}`);
      }
    },
    loadManifestMods() {
      invoke('query_mod_manifest').then((mods) => {
        this.manifest_mods = mods;
        this.generateSelectedModIds = mods.filter((mod) => mod.enabled).map((mod) => mod.id);
        this.generateSelectedModSources = this.defaultGenerateModSources();
        info('Loaded Mod Manifest Entries: ' + JSON.stringify(this.manifest_mods));
      }).catch((err) => {
        error('Failed to load mod manifest: ' + err);
      });
    },
    loadAppVersion() {
      invoke('query_app_version').then((version) => {
        this.appVersion = version;
      }).catch((err) => {
        error('Failed to load app version: ' + err);
      });
    },
    loadAppBuildUid() {
      invoke('query_app_build_uid').then((uid) => {
        this.appBuildUid = uid;
      }).catch((err) => {
        error('Failed to load app build UID: ' + err);
      });
    },
    async bindOptimizationStatusEvents() {
      this.unlistenOptimizationStatus = await listen('optimization-status', (event) => {
        if (event.payload && event.payload.message) {
          this.backendOptimizationMessage = event.payload.message;
        }
        if (event.payload) {
          const current = Number(event.payload.current || 0);
          const total = Number(event.payload.total || 0);
          if (total > 0) {
            this.optimizationProgress = Math.min(100, Math.max(0, (current / total) * 100));
          }
        }
      });
    },
    openGenerateSdCardDialog() {
      this.showGenerateSdCard = true;
      if (this.generateSelectedModIds.length === 0 && this.manifest_mods.length > 0) {
        this.generateSelectedModIds = this.manifest_mods.filter((mod) => mod.enabled).map((mod) => mod.id);
      }
      this.generateSelectedModSources = this.defaultGenerateModSources();
    },
    defaultGenerateModSourceOption(modId) {
      const mod = this.manifest_mods.find((entry) => entry.id === modId);
      if (!mod) {
        return null;
      }
      if (mod.default_source_option) {
        return mod.default_source_option;
      }
      return (mod.source_options && mod.source_options.length > 0) ? mod.source_options[0] : null;
    },
    defaultGenerateModSources() {
      const selected = {};
      for (const mod of this.manifest_mods) {
        selected[mod.id] = this.generateSelectedModSources[mod.id] || this.defaultGenerateModSourceOption(mod.id);
      }
      return selected;
    },
    onGenerateSelectedModIdsUpdated(ids) {
      this.generateSelectedModIds = ids;
    },
    onGenerateSelectedModSourcesUpdated(sourceMap) {
      this.generateSelectedModSources = sourceMap;
    },
    async generateSdCardFolder() {
      if (this.isOptimizing || this.generateSelectedModIds.length === 0) {
        return;
      }

      this.isOptimizing = true;
      this.currentOptimization = 'GenerateSdCard';
      this.backendOptimizationMessage = null;
      this.optimizationProgress = null;
      try {
        const cleanupOptions = [
          { EnableMods: [...this.generateSelectedModIds] },
          {
            SelectModSources: this.generateSelectedModIds.map((id) => ({
              modId: id,
              sourceOption: this.generateSelectedModSources[id] || this.defaultGenerateModSourceOption(id),
            })),
          },
        ];
        const outputPath = await invoke('generate_sdcard_folder', { cleanupOptions });
        this.showGenerateSdCard = false;
        this.showSnackbar(`Generated SDCard folder: ${outputPath}`, 5000, 'green');
      } catch (err) {
        error(err);
        this.showSnackbar('Error generating SDCard folder', 3500, 'red');
      } finally {
        this.currentOptimization = null;
        this.backendOptimizationMessage = null;
        this.optimizationProgress = null;
        this.isOptimizing = false;
      }
    },
    async refreshLogs() {
      this.logsLoading = true;
      try {
        this.logsContent = await invoke('query_logs');
      } catch (err) {
        this.logsContent = `Unable to load logs.\n${err}`;
        error('Failed to load logs: ' + err);
      } finally {
        this.logsLoading = false;
        await this.scrollLogsToBottom();
      }
    },
    async scrollLogsToBottom() {
      await nextTick();
      const logsPre = this.$refs.logsPre;
      if (logsPre && logsPre.parentElement) {
        logsPre.parentElement.scrollTop = logsPre.parentElement.scrollHeight;
      }
    },
    async openLogsDialog() {
      this.showLogs = true;
      await this.refreshLogs();
    },
    openLogsFolder() {
      invoke('open_logs_folder').catch((err) => {
        error('Unable to open logs folder: ' + err);
        this.showSnackbar('Unable to open logs folder', 3000, 'red');
      });
    },
    updateUserStatus() {
      invoke('get_user_status', { userProfile: this.selected_profile }).then((status) => {
        info('Updated User Status: ' + JSON.stringify(status));
        this.user_status.settings_optimized = status.settings_optimized;
        this.user_status.mods_optimized = status.mods_optimized;
        this.user_status.save_optimized = status.save_optimized;
      }).catch((err) => {
        error(err);
      })
    },
    profileSelected(profile) {
      this.selected_profile = profile;
      this.updateUserStatus()
      info('Selected Profile: ' + JSON.stringify(this.selected_profile));
    },
    optUpdated(key, enabled, options) {
      this.selected_opts[key].enabled = enabled;
      this.selected_opts[key].options = options;
      info('Optimization Updated: ' + JSON.stringify(this.selected_opts));
    },
    async optimizeSelected() {
      if (this.isOptimizing) {
        return;
      }

      this.isOptimizing = true;
      this.backendOptimizationMessage = null;
      this.optimizationProgress = null;
      try {
        for (const [key, data] of Object.entries(this.selected_opts)) {
          if (data.enabled) {
            this.currentOptimization = key;
            if (key !== 'Mods') {
              this.optimizationProgress = null;
            }
            const args = { userProfile: this.selected_profile, optimization: key, cleanupOptions: data.options };
            await invoke('apply_optimization', args).then(() => {
              info('Optimization Applied: ' + JSON.stringify(args));
              this.showSnackbar('Optimization Applied Successfully: ' + key, 3000, "green");
              this.updateUserStatus();
            }).catch((err) => {
              error(err);
              this.showSnackbar('Error Applying Optimization: ' + key, 3000, "red");
            });
          }
        }
      } finally {
        this.currentOptimization = null;
        this.backendOptimizationMessage = null;
        this.optimizationProgress = null;
        this.isOptimizing = false;
      }
    },
    calcSnackbarMargin(i) {
      return (i * 60) + 'px'
    },
    showSnackbar(message, timeout, color) {
      const snackbar = { show: true, text: message, timeout: timeout, color: color }
      this.snackbars.push(snackbar);
      setTimeout(() => this.hideSnackbar(this.snackbars.length - 1), timeout);
    },
    hideSnackbar(i) {
      this.snackbars.splice(i, 1);
    }
  }
}
</script>

<style>
.profile-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.profile-trigger {
  flex-shrink: 0;
  min-width: 500px;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.logs-content {
  background: #0f0f0f;
  max-height: min(300px, 45vh);
  overflow: auto;
}

.logs-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 160px;
}

.logs-pre {
  margin: 0;
  padding: 12px;
  font-family: "Courier New", monospace;
  font-size: 12px;
  line-height: 1.35;
  white-space: pre-wrap;
  word-break: break-word;
}

html {
  overflow: hidden !important;
  scrollbar-width: none;
  -ms-overflow-style: none;
  overscroll-behavior: none;
}

html::-webkit-scrollbar {
  display: none;
  width: 0;
  height: 0;
}

@media (max-width: 640px) {
  .profile-row {
    align-items: flex-start;
    flex-direction: column;
  }

  .profile-trigger {
    min-width: 300px;
  }
}
</style>
