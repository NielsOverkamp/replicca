local r = http.get("https://files.nielsoverkamp.com/replicca/install.lua")
loadstring(r.readAll())()
