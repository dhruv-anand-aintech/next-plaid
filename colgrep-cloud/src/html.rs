//! Static HTML pages for the web UI

pub const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><title>ColGREP Cloud</title><meta charset="utf-8">
<style>body{font-family:system-ui;max-width:600px;margin:3rem auto;padding:0 1rem}nav a{margin-right:1rem}</style>
</head>
<body>
<h1>ColGREP Cloud</h1>
<p>Semantic code search in the cloud. Index your codebases and search by meaning.</p>
<nav>
<a href="/login">Log in</a>
<a href="/register">Register</a>
<a href="/dashboard">Dashboard</a>
</nav>
</body>
</html>"#;

pub const LOGIN_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><title>Log in - ColGREP Cloud</title><meta charset="utf-8">
<style>body{font-family:system-ui;max-width:400px;margin:3rem auto;padding:0 1rem}input{display:block;width:100%;margin:0.5rem 0;padding:0.5rem}button{margin-top:1rem;padding:0.5rem 1rem}</style>
</head>
<body>
<h1>Log in</h1>
<form id="form">
<input type="email" name="email" placeholder="Email" required>
<input type="password" name="password" placeholder="Password" required>
<button type="submit">Log in</button>
</form>
<p id="msg"></p>
<script>
document.getElementById('form').onsubmit=async e=>{
e.preventDefault();
const fd=new FormData(e.target);
const r=await fetch('/api/login',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({email:fd.get('email'),password:fd.get('password')})});
const j=await r.json();
if(r.ok){localStorage.setItem('token',j.token);location.href='/dashboard'} else{document.getElementById('msg').textContent=j.error||'Login failed'}}
</script>
</body>
</html>"#;

pub const REGISTER_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><title>Register - ColGREP Cloud</title><meta charset="utf-8">
<style>body{font-family:system-ui;max-width:400px;margin:3rem auto;padding:0 1rem}input{display:block;width:100%;margin:0.5rem 0;padding:0.5rem}button{margin-top:1rem;padding:0.5rem 1rem}</style>
</head>
<body>
<h1>Register</h1>
<form id="form">
<input type="email" name="email" placeholder="Email" required>
<input type="password" name="password" placeholder="Password (min 8 chars)" required minlength="8">
<button type="submit">Register</button>
</form>
<p id="msg"></p>
<script>
document.getElementById('form').onsubmit=async e=>{
e.preventDefault();
const fd=new FormData(e.target);
const r=await fetch('/api/register',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({email:fd.get('email'),password:fd.get('password')})});
const j=await r.json();
if(r.ok){localStorage.setItem('token',j.token);location.href='/dashboard'} else{document.getElementById('msg').textContent=j.error||'Registration failed'}}
</script>
</body>
</html>"#;

pub const DASHBOARD_HTML: &str = r##"<!DOCTYPE html>
<html>
<head><title>Dashboard - ColGREP Cloud</title><meta charset="utf-8">
<style>body{font-family:system-ui;max-width:800px;margin:3rem auto;padding:0 1rem}a{margin-right:1rem}.cb{border:1px solid #ccc;padding:1rem;margin:1rem 0;border-radius:4px}</style>
</head>
<body>
<h1>Your Codebases</h1>
<nav><a href="/">Home</a><a href="/logout">Log out</a></nav>
<div id="list">Loading...</div>
<script>
const token=localStorage.getItem('token');
if(!token){location.href='/login';}
fetch('/api/codebases',{headers:{'Authorization':'Bearer '+token}}).then(r=>{
if(r.status===401){location.href='/login';return}
return r.json()}).then(data=>{
document.getElementById('list').innerHTML=Array.isArray(data)&&data.length
? data.map(c=>'<div class="cb"><b>'+c.name+'</b> '+c.file_count+' files, '+c.code_unit_count+' units</div>').join('')
: '<p>No codebases yet. Use the API to create and upload.</p>'}).catch(()=>document.getElementById('list').textContent='Error loading');
</script>
</body>
</html>"##;
