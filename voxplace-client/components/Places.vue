<template>
  <div class="w-full flex flex-col h-full items-center">
    <div class="bg-black w-[33rem] lg:w-[65rem] 2xl:w-[97rem] h-0.5 mt-4"></div>
    <div class="overflow-y-scroll overflow-hidden block h-full scrollbar-hide">
      <div class="grid grid-cols-1 2xl:grid-cols-2 gap-16 p-4">
        <Place
            v-for="place in places"
            :name="place.name"
            :place_id="place.place_id"
            :voxel_id="place.voxel_id"
            online/>
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
  places.value = await res.json();
  console.log(places);
}
</script>