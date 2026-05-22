<script setup lang="ts">
import OptimizationSettings from '../../components/OptimizationSettings.vue'

defineEmits<{
  (e: 'saved'): void
}>()
</script>

<template>
  <div class="optimization-page">
    <OptimizationSettings @saved="$emit('saved')" />
  </div>
</template>

<style scoped>
</style>
