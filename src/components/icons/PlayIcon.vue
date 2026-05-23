<script setup lang="ts">
import IconBase from './IconBase.vue'

defineProps<{
  size?: number
}>()
</script>

<template>
  <IconBase :size="size">
    <polygon points="5 3 19 12 5 21 5 3" />
  </IconBase>
</template>
