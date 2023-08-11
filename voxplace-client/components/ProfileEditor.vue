<template>
  <div class="bg-white border-2 border-black rounded-2xl shadow-custom h-fit p-8">
    <NuxtLink :to="'/editor/' + profile.voxel_id" class="w-full h-full">
      <div class="bg-white border-2 border-black rounded-2xl w-[24rem] h-[24rem] flex justify-between items-center shadow-custom relative cursor-pointer">
        <VoxelPreview :voxel_id="profile.voxel_id"/>
        <div class="bg-neutral-200 border-black border-t-2 rounded-b-2xl w-full h-10 absolute bottom-0 flex items-center justify-between">
          <p class="ml-8 text-xl">Go to editor</p>
        </div>
      </div>
    </NuxtLink>
    <div class="mt-8 flex-col">
      <div v-if="!(changeUsername || changeEmail || changePassword)" class="flex flex-col">
        <Button text="Change Username" @click="changeUsername = true" class="bg-white hover:bg-neutral-300 w-full"/>
        <Button text="Change Email" @click="changeEmail = true" class="bg-white hover:bg-neutral-300 w-full mt-4"/>
        <Button text="Change Password" @click="changePassword = true" class="bg-white hover:bg-neutral-300 w-full mt-4"/>
      </div>
      <div v-if="changeUsername" class="flex flex-col">
        <p class="text-lg">Password :</p>
        <input v-model="password" type="password" placeholder="password" class="w-full bg-white border-2 border-black rounded-2xl px-4 py-1"/>
        <div class="mt-2 h-6 flex items-start w-full">
          <p v-if="wrongCredentials" class="text-red-500">Wrong password or session error</p>
        </div>
        <p class="text-lg">Current username :</p>
        <input :placeholder="profile.username" type="text" disabled class="w-full bg-white border-2 border-neutral-500 rounded-2xl px-4 py-1"/>
        <p class="mt-6 text-lg">New username :</p>
        <input v-model="newUsername" type="text" placeholder="New Username" class="w-full bg-white border-2 border-black rounded-2xl px-4 py-1"/>
        <div class="mt-2 h-6 flex items-start w-full">
          <p v-if="usernameTooShort" class="text-red-500">Username too short</p>
        </div>
      </div>
      <div v-if="changeEmail" class="flex flex-col">
        <p class="text-lg">Password :</p>
        <input v-model="password" type="password" placeholder="password" class="w-full bg-white border-2 border-black rounded-2xl px-4 py-1"/>
        <div class="mt-2 h-6 flex items-start w-full">
          <p v-if="wrongCredentials" class="text-red-500">Wrong password or session error</p>
        </div>
        <p class="text-lg">Current email :</p>
        <input :placeholder="profile.email" type="text" disabled class="w-full bg-white border-2 border-neutral-500 rounded-2xl px-4 py-1"/>
        <p class="mt-6 text-lg">New email :</p>
        <input v-model="newEmail" type="text" placeholder="New Email" class="w-full bg-white border-2 border-black rounded-2xl px-4 py-1"/>
        <div class="mt-2 h-6 flex items-start w-full">
          <p v-if="emailNotValid" class="text-red-500">Email not valid</p>
        </div>
      </div>
      <div v-if="changePassword" class="flex flex-col">
        <p class="text-lg">Password :</p>
        <input v-model="password" type="password" placeholder="password" class="w-full bg-white border-2 border-black rounded-2xl px-4 py-1"/>
        <div class="mt-2 h-6 flex items-start w-full">
          <p v-if="wrongCredentials" class="text-red-500">Wrong password or session error</p>
        </div>
        <p class="text-lg">New password :</p>
        <input v-model="newPassword" type="password" placeholder="New Password" class="w-full bg-white border-2 border-black rounded-2xl px-4 py-1"/>
        <div class="mt-2 h-6 flex items-start w-full">
          <p v-if="passwordTooShort" class="text-red-500">Password too short</p>
          <p v-if="passwordNotMatch" class="text-red-500">Passwords don't match</p>
        </div>
        <p class="text-lg">Repeat new password :</p>
        <input v-model="newRepeatPassword" type="password" placeholder="New Password" class="w-full bg-white border-2 border-black rounded-2xl px-4 py-1"/>
        <div class="mt-2 h-6 flex items-start w-full">
          <p v-if="passwordTooShort" class="text-red-500">Password too short</p>
          <p v-if="passwordNotMatch" class="text-red-500">Passwords don't match</p>
        </div>
      </div>
      <div class="flex justify-between">
        <Button text="Cancel" @click="cancel" class="bg-white hover:bg-neutral-300 mt-2 w-32"/>
        <Button text="Save" @click="save" v-if="changeUsername || changeEmail || changePassword" class="bg-blue-300 hover:bg-blue-400 mt-2 w-32"/>
      </div>
    </div>
  </div>
</template>

<script setup>
let changeUsername = ref(false);
let changeEmail = ref(false);
let changePassword = ref(false);

let password = ref('');
let newUsername = ref('');
let newEmail = ref('');
let newPassword = ref('');
let newRepeatPassword = ref('');

let usernameTooShort = ref(false);
let emailNotValid = ref(false);
let passwordNotMatch = ref(false);
let passwordTooShort = ref(false);
let wrongCredentials = ref(false);

const emit = defineEmits(['close'])

const props = defineProps({
  profile: {
    type: Object,
    required: true
  }
})

function cancel(){
  if(changeUsername.value || changeEmail.value || changePassword.value){
    changeUsername.value = false;
    changeEmail.value = false;
    changePassword.value = false;
  } else {
    emit('close');
  }
  password.value = '';
  newUsername.value = '';
  newEmail.value = '';
  newPassword.value = '';
  newRepeatPassword.value = '';
  usernameTooShort.value = false;
  emailNotValid.value = false;
  passwordNotMatch.value = false;
  passwordTooShort.value = false;
  wrongCredentials.value = false;
}

function save(){
  usernameTooShort.value = false;
  emailNotValid.value = false;
  passwordNotMatch.value = false;
  passwordTooShort.value = false;
  wrongCredentials.value = false;

  if (changeUsername.value) {
    saveUsername();
  } else if (changeEmail.value) {
    saveEmail();
  } else if (changePassword.value) {
    savePassword();
  }
}

async function saveUsername() {
  if(newUsername.value.length < 3) {
    usernameTooShort.value = true;
    return;
  }

  let token = localStorage.getItem('token');

  const res = await fetch(`http://${window.location.hostname}:8000/api/user/edit`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token
    },
    body: JSON.stringify({
      username: props.profile.username,
      password: password.value,
      newUsername: newUsername.value,
      newEmail: "",
      newPassword: "",
    })
  });
  if(res.ok){
    changeUsername.value = false;
    password.value = '';
    props.profile.username = newUsername.value;
    newUsername.value = '';
  } else {
    wrongCredentials.value = true;
    console.error(await res.text());
  }
}

async function saveEmail() {
  const emailRegex = /\S+@\S+\.\S+/;
  if(!emailRegex.test(newEmail.value)) {
    emailNotValid.value = true;
    return;
  }

  let token = localStorage.getItem('token');

  const res = await fetch(`http://${window.location.hostname}:8000/api/user/edit`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token
    },
    body: JSON.stringify({
      username: props.profile.username,
      password: password.value,
      newUsername: "",
      newEmail: newEmail.value,
      newPassword: "",
    })
  });
  if(res.ok){
    changeEmail.value = false;
    password.value = '';
    props.profile.email = newEmail.value;
    newEmail.value = '';
  } else {
    wrongCredentials.value = true;
    console.error(await res.text());
  }
}

async function savePassword() {
  if(newPassword.value.length < 8) {
    passwordTooShort.value = true;
    return;
  }

  if(newPassword.value !== newRepeatPassword.value) {
    passwordNotMatch.value = true;
    return;
  }

  console.log(newPassword.value, newRepeatPassword.value);

  let token = localStorage.getItem('token');

  const res = await fetch(`http://${window.location.hostname}:8000/api/user/edit`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token
    },
    body: JSON.stringify({
      username: props.profile.username,
      password: password.value,
      newUsername: "",
      newEmail: "",
      newPassword: newPassword.value
    })
  });
  if(res.ok){
    changePassword.value = false;
    password.value = '';
    newPassword.value = '';
    newRepeatPassword.value = '';
  } else {
    wrongCredentials.value = true;
    console.error(await res.text());
  }
}
</script>