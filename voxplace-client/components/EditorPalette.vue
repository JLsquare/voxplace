<template>
  <div class="w-full h-full border-2 border-black rounded-2xl p-4 flex shadow-custom">
    <div class="flex items-center w-full">
      <div class="bg-white h-full overflow-hidden grid grid-cols-8 gap-1">
        <div
            v-for="(color, index) in palette"
            :key="index"
            @click="selectedColor = color; selectedIndex = index"
            :class="index === selectedIndex ? 'border-2 border-black' : 'border border-black'"
            class="w-6 h-6 rounded-lg cursor-pointer hover:border-2"
            :style="{ backgroundColor: color }"
        />
      </div>
    </div>
    <div class="w-full h-full border-2 border-black rounded-2xl flex flex-col p-4">
      <p class="text-2xl">Tool :</p>
      <div class="grid grid-cols-4 gap-4 mt-2">
        <div
            v-for="(tool, index) in tools"
            :key="index"
            @click="selectedTool = index"
            :class="selectedTool === index ? 'bg-neutral-200' : 'bg-white'"
            class="h-12 w-12 flex items-center hover:bg-neutral-300 justify-center border-2 shadow-custom border-black rounded-lg p-1 cursor-pointer"
        >
          <img :src="tool"/>
        </div>
      </div>
      <p class="text-2xl mt-8">Action :</p>
      <div class="flex flex-col space-y-4 w-full items-center">
        <Button
            v-for="(action, index) in actions"
            :key="index"
            @click="selectedAction = index"
            :class="selectedAction === index ? 'bg-neutral-200' : 'bg-white'"
            :text="action"
            class="hover:bg-neutral-300 mt-2 w-32"/>
      </div>
      <div class="flex-grow"></div>
      <div class="flex items-center justify-between w-full">
        <Button text="Cancel" class="bg-white hover:bg-neutral-300 w-28"/>
        <Button text="Save" @click="$emit('save')" class="bg-blue-300 hover:bg-blue-400 w-28"/>
      </div>
    </div>
  </div>
</template>

<script setup>
const route = useRoute();
let palette = ref([]);

let selectedColor = ref('');
let selectedIndex = ref(-1);
let selectedTool = ref(-1);
let selectedAction = ref(-1);

const tools = [
  '/point-tool.svg',
  '/line-tool.svg',
  '/box-tool.svg',
  '/extrude-tool.svg',
  '/pick-tool.svg',
  '/bucket-tool.svg',
  '/select-tool.svg'
];

const actions = ['Attach', 'Erase', 'Paint'];

onMounted(() => {
  getPalette();
});

async function getPalette() {
  const response = await fetch(`http://${window.location.hostname}:8000/api/voxel/palette/${route.params.id}`, {
    method: 'GET',
    headers: {
      'Content-Type': 'application/json',
    },
  });

  if (!response.ok) {
    const message = await response.text();
    console.log(message);
  } else {
    palette.value = await response.json();
    for(let i = palette.value.length; i < 256; i++) {
      palette.value.push('#ffffff');
    }
  }
}

const emit = defineEmits(['selection-changed']);

watch([selectedIndex, selectedTool, selectedAction], () => {
  emit('selection-changed', {
    color: selectedIndex.value,
    tool: selectedTool.value,
    action: selectedAction.value
  });
});
</script>
