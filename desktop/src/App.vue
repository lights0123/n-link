<template>
  <div id="app" class="h-screen">
    <router-view/>
  </div>
</template>

<style lang="scss">
@import '~n-link-core/assets/tailwind.css';
#app {
  user-select: none;
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #2c3e50;
}
</style>
