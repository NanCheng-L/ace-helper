<script setup lang="ts">
import IconBase from './IconBase.vue'

defineProps<{
  size?: number
}>()
</script>

<template>
  <IconBase :size="size">
    <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
  </IconBase>
</template>
