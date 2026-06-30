<template>
  <v-card class="mx-auto opt-card-main" :title="cardTitle" :subtitle="cardSubtitle" @click="toggle">
    <template v-slot:append>
      <StatusIcon :isCorrect="isOptimized" :correctMessage="'Optimized'" :incorrectMessage="'Not Optimized'"
        :location="'left'" />
    </template>
    <template v-slot:prepend>
      <v-icon :icon="cardDisplayIcon"></v-icon>
    </template>
    <v-dialog v-if="hasCleanupPopup" v-model="showCleanup" max-width="760">
      <template v-slot:activator="{ props }">
        <v-btn :disabled="!isSelected" color="purple-lighten-2" v-bind="props" append-icon="mdi-cog"
          variant="text" size="small" @click.stop>
          Configure
        </v-btn>
      </template>

      <v-card class="mods-config-dialog" title="Mods Configuration">
        <v-card-subtitle>Choose cleanup options and which mods to install for this run.</v-card-subtitle>
        <v-card-text>
          <v-sheet v-if="cleanupOptions && cleanupOptions.length > 0" class="config-section" rounded="lg" border>
            <div class="section-title">Cleanup Options</div>
            <div class="cleanup-options-wrap">
              <v-switch
                v-for="option in cleanupOptions"
                :key="option.id"
                class="cleanup-switch"
                color="purple-lighten-2"
                v-model="selectedOptions"
                :value="option.id"
                :label="option.label"
                @change="optionsUpdated"
                hide-details
                density="compact"
              />
            </div>
          </v-sheet>

          <ModSelectionChecklist
            v-if="modOptions && modOptions.length > 0"
            section-title="Mods To Install"
            :mods="modOptions"
            :selected-mod-ids="selectedModIds"
            :selected-mod-sources="selectedModSources"
            @update:selected-mod-ids="onSelectedModIdsUpdated"
            @update:selected-mod-sources="onSelectedModSourcesUpdated"
          />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn color="purple-lighten-2" variant="text" @click="showCleanup = false">Done</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-checkbox
      color="purple-lighten-2"
      class="opt-card-sub"
      :model-value="isSelected"
      @update:model-value="onCheckboxUpdated"
      @click.stop
    />
  </v-card>
</template>

<script>
export default {
  props: {
    cardTitle: String,
    cardSubtitle: String,
    cardDisplayIcon: String,
    isOptimized: Boolean,
    cleanupOptions: Array,
    modOptions: Array,
  },
  data() {
    return {
      isSelected: true,
      showCleanup: false,
      selectedOptions: [],
      selectedModIds: [],
      selectedModSources: {},
    };
  },
  computed: {
    hasCleanupPopup() {
      const hasCleanupOptions = this.cleanupOptions && this.cleanupOptions.length > 0;
      const hasModOptions = this.modOptions && this.modOptions.length > 0;
      return hasCleanupOptions || hasModOptions;
    },
  },
  watch: {
    isOptimized: {
      immediate: true, 
      handler (newVal) {
      this.isSelected = !newVal;
      this.optionsUpdated();
      }
    },
    modOptions: {
      immediate: true,
      handler(newVal) {
        const options = newVal || [];
        const ids = options.map((entry) => entry.id);
        if (ids.length === 0) {
          this.selectedModIds = [];
          this.selectedModSources = {};
          this.optionsUpdated();
          return;
        }

        const retained = this.selectedModIds.filter((id) => ids.includes(id));
        this.selectedModIds = retained.length > 0 ? retained : this.defaultSelectedModIds();
        this.selectedModSources = this.defaultSelectedModSources();
        this.optionsUpdated();
      },
    }
  },
  methods: {
    currentSelectionPayload() {
      const options = [...this.selectedOptions];
      if (this.modOptions && this.modOptions.length > 0) {
        options.push({ EnableMods: [...this.selectedModIds] });
        options.push({ SelectModSources: this.selectedModIds.map((id) => ({ modId: id, sourceOption: this.selectedModSources[id] || this.defaultModSourceOption(id) })) });
      }
      return options;
    },
    defaultSelectedModIds() {
      return (this.modOptions || [])
        .filter((entry) => entry.enabled)
        .map((entry) => entry.id);
    },
    onSelectedModIdsUpdated(ids) {
      this.selectedModIds = ids;
      this.optionsUpdated();
    },
    onSelectedModSourcesUpdated(sourceMap) {
      this.selectedModSources = sourceMap;
      this.optionsUpdated();
    },
    defaultModSourceOption(modId) {
      const mod = (this.modOptions || []).find((entry) => entry.id === modId);
      if (!mod) {
        return null;
      }
      if (mod.default_source_option) {
        return mod.default_source_option;
      }
      return (mod.source_options && mod.source_options.length > 0) ? mod.source_options[0] : null;
    },
    defaultSelectedModSources() {
      const selected = {};
      for (const mod of (this.modOptions || [])) {
        selected[mod.id] = this.selectedModSources[mod.id] || this.defaultModSourceOption(mod.id);
      }
      return selected;
    },
    setSelection(nextSelected) {
      this.isSelected = nextSelected;
      if (!this.isSelected) {
        this.showCleanup = false;
        this.selectedOptions = [];
        this.selectedModIds = this.defaultSelectedModIds();
        this.selectedModSources = this.defaultSelectedModSources();
      }
      this.optionsUpdated();
    },
    onCheckboxUpdated(nextSelected) {
      this.setSelection(nextSelected);
    },
    toggle() {
      this.setSelection(!this.isSelected);
    },
    optionsUpdated() {
      this.$emit('updated', this.isSelected, this.currentSelectionPayload());
    }
  }
};
</script>

<style scoped>
.opt-card-main {
  position: relative;
  padding: 20px;
  margin: 0px;
}

.opt-card-sub {
  position: absolute;
  top: 0;
  left: 0;
}

.mods-config-dialog {
  border-radius: 14px;
}

.config-section {
  padding: 14px;
}

.section-title {
  font-size: 0.92rem;
  font-weight: 600;
  margin-bottom: 8px;
}

.cleanup-options-wrap {
  padding-left: 6px;
}

.cleanup-switch {
  margin: 0;
}
</style>
