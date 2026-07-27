<template>
  <div id="app" class="h-screen">
    <router-view/>
  </div>
</template>

<style lang="scss">
@import 'n-link-core/assets/tailwind.css';

#app {
  user-select: none;
  font-family: 'Inter', theme('fontFamily.sans');
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  background-color: var(--surface);
  color: var(--fg);
}
</style>
