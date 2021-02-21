local conf_path = ".cfg"
local remote = "replicca.mc.nielsoverkamp.com/api"

local MODES = {
    EVAL="EVAL",
    FULL_DATA="FULL_DATA",
}

local id = nil
local mode = nil
if fs.exists(conf_path) then
    local f = fs.open(conf_path, "r")
    id = f.readLine()
    id = tonumber(id)

    mode = f.readLine()
end

if id == nil then
    local r = http.get("http://"..remote .. "/newId")
    id = r.readAll()
    id = tonumber(id)
end

if id == nil then
    error("Could not find local id file and could not get new one from remote")
    return
end

if mode == nil then
    mode = MODES.FULL_DATA
end

local ws, err = http.websocket("ws://"..remote .. "/ws")

if not ws then
    error(err)
    return
end

while true do
    if mode == MODES.EVAL then

    end
    print(ws.receive())

end
