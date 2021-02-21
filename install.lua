local url = "https://files.nielsoverkamp.com/replicca"


local r = http.get(url.."/startup.lua")
if r == nil then
    error("Could not get startup script from "..url.." rebooting in 5...")
    os.sleep(5)
    os.reboot()
    return
end
local f = fs.open("startup.lua", "w")
f.write(r.readAll())
f.close()
r.close()

print("updated startup.lua, rebooting in 5")
for i=4,0,-1 do
    os.sleep(1)
    print(i)
end
os.reboot()
