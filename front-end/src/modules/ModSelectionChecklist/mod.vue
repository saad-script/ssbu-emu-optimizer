<template>
  <v-sheet v-if="mods && mods.length > 0" class="config-section" rounded="lg" border>
    <div class="section-header">
      <div class="section-title">{{ sectionTitle }}</div>
      <div class="selection-actions">
        <v-btn size="small" variant="text" color="purple-lighten-2" @click="toggleAllMods">Toggle All</v-btn>
        <v-chip size="small" color="purple-lighten-3" variant="flat">{{ selectedModIds.length }} / {{ mods.length }} selected</v-chip>
      </div>
    </div>

    <v-list class="mod-selection-list" bg-color="transparent" density="compact">
      <v-list-item v-for="mod in mods" :key="mod.id" @click="toggleModSelection(mod.id)">
        <template v-slot:prepend>
          <v-checkbox-btn
            color="purple-lighten-2"
            :model-value="selectedModIds.includes(mod.id)"
            @click.stop="toggleModSelection(mod.id)"
          />
        </template>
        <v-list-item-title>
          <div class="mod-title-row">
            <div class="mod-name-wrap">
              <span>{{ mod.name }}</span>
              <v-tooltip v-if="mod.description" location="top" max-width="460">
                <template v-slot:activator="{ props }">
                  <v-icon
                    v-bind="props"
                    class="mod-description-icon"
                    size="16"
                    icon="mdi-information-outline"
                  />
                </template>
                <span>{{ mod.description }}</span>
              </v-tooltip>
            </div>
            <v-select
              v-if="mod.source_options && mod.source_options.length > 1"
              class="mod-source-select-mini"
              density="compact"
              hide-details
              :items="sourceOptionItems(mod)"
              item-title="title"
              item-value="value"
              :model-value="selectedModSources[mod.id]"
              @update:model-value="(value) => updateModSourceOption(mod.id, value)"
              @click.stop
            />
          </div>
        </v-list-item-title>
      </v-list-item>
    </v-list>
  </v-sheet>
</template>

<script>
export default {
  props: {
    sectionTitle: {
      type: String,
      default: 'Mods',
    },
    mods: {
      type: Array,
      default: () => [],
    },
    selectedModIds: {
      type: Array,
      default: () => [],
    },
    selectedModSources: {
      type: Object,
      default: () => ({}),
    },
  },
  emits: ['update:selectedModIds', 'update:selectedModSources'],
  methods: {
    sourceOptionItems(mod) {
      return (mod.source_options || []).map((option) => ({
        title: `(${option})`,
        value: option,
      }));
    },
    defaultModSourceOption(modId) {
      const mod = (this.mods || []).find((entry) => entry.id === modId);
      if (!mod) {
        return null;
      }
      if (mod.default_source_option) {
        return mod.default_source_option;
      }
      return (mod.source_options && mod.source_options.length > 0) ? mod.source_options[0] : null;
    },
    normalizedSources(nextSources) {
      const normalized = { ...nextSources };
      for (const mod of (this.mods || [])) {
        normalized[mod.id] = normalized[mod.id] || this.defaultModSourceOption(mod.id);
      }
      return normalized;
    },
    toggleAllMods() {
      const nextIds = this.selectedModIds.length === this.mods.length
        ? []
        : this.mods.map((mod) => mod.id);
      this.$emit('update:selectedModIds', nextIds);
      this.$emit('update:selectedModSources', this.normalizedSources(this.selectedModSources));
    },
    toggleModSelection(modId) {
      const nextIds = this.selectedModIds.includes(modId)
        ? this.selectedModIds.filter((id) => id !== modId)
        : [...this.selectedModIds, modId];
      const nextSources = {
        ...this.selectedModSources,
        [modId]: this.selectedModSources[modId] || this.defaultModSourceOption(modId),
      };
      this.$emit('update:selectedModIds', nextIds);
      this.$emit('update:selectedModSources', this.normalizedSources(nextSources));
    },
    updateModSourceOption(modId, optionName) {
      this.$emit('update:selectedModSources', {
        ...this.selectedModSources,
        [modId]: optionName,
      });
    },
  },
};
</script>

<style scoped>
.config-section {
  padding: 14px;
}

.section-title {
  font-size: 0.92rem;
  font-weight: 600;
  margin-bottom: 8px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 8px;
}

.selection-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.selection-actions :deep(.v-chip) {
  margin-right: 2px;
}

.selection-actions :deep(.v-btn) {
  min-width: 0;
  padding-left: 8px;
  padding-right: 8px;
}

.mod-title-row {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 15px;
}

.mod-source-select-mini {
  width: 150px !important;
  min-width: 150px;
  max-width: 150px;
  flex: 0 0 auto !important;
}

.mod-source-select-mini :deep(.v-select__selection-text) {
  overflow: visible;
  text-overflow: clip;
  white-space: nowrap;
}

@media (max-width: 640px) {
  .section-header {
    align-items: flex-start;
    flex-direction: column;
  }
}

.mod-name-wrap {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.mod-description-icon {
  opacity: 0.72;
}
</style>
