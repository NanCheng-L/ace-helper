<script setup lang="ts">
import IconBase from './IconBase.vue'

defineProps<{
  size?: number
}>()
</script>

<template>
  <IconBase :size="size">
    <polyline points="20 6 9 17 4 12" />
  </IconBase>
</template>
