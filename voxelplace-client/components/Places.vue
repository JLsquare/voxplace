<template>
  <div class="w-full flex flex-col h-full items-center">
    <div class="bg-black w-[33rem] lg:w-[65rem] 2xl:w-[97rem] h-0.5 mt-4"></div>
    <div class="overflow-y-scroll overflow-hidden block h-full scrollbar-hide">
      <div class="grid grid-cols-1 lg:grid-cols-2 2xl:grid-cols-3 gap-8 p-4">
        <PlacePost v-for="place in places" :key="place.id" :name="place.name" :id="place.id" online/>
      </div>
    </div>
  </div>
</template>

<script setup>
let places = ref([]);

onMounted(() => {
  getPlaces();
});

async function getPlaces() {
  const res = await fetch(`http://${window.location.hostname}:8000/api/place/infos`);
  const data = await res.json();
  for(let i = 0; i < data.length; i++) {
    places.value.push({name: data[i].name, id: data[i].id});
  }
}
</script>