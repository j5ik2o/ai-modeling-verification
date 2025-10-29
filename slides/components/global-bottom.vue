<template>
  <footer class="absolute bottom-0 left-0 right-0 p-2">
    {{ $slidev.nav.currentPage }} / {{ $slidev.nav.total }}
  </footer>
</template>

<style>
.slidev-page-number-global {
  position: fixed;
  bottom: 1.2rem;
  right: 1.5rem;
  font-size: 0.85rem;
  color: #888;
  font-family: 'Noto Sans JP', sans-serif;
  z-index: 100;
  opacity: 0.8;
}
</style>
