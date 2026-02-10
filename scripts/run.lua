print("Starting")

Name = ""
KNOWNBLOCKS = {
    "computercraft:turtle_normal",
    "computercraft:turtle_advanced",
}
READY = {
    type = "ready",
}

local function hasValue(table, value)
    for _, v in ipairs(table) do
        if v == value then
            return true
        end
    end

    return false
end

local function getPosition()
    local coords = {}
    local handle = io.open("/position", "r")
    if handle == nil then
        return nil
    end

    for line in handle:lines() do
        table.insert(coords, line)
    end

    handle:close()

    if table.getn(coords) ~= 4 then
        return nil
    end

    return {
        x = tonumber(coords[1]),
        y = tonumber(coords[2]),
        z = tonumber(coords[3]),
        heading = coords[4],
    }
end

local function setPosition(coords)
    local handle = fs.open("position", "w")
    local formatted = coords.x .. "\n" .. coords.y .. "\n" .. coords.z .. "\n" .. coords.heading
    handle.write(formatted)
    handle.close()
end

local function updatePosition(position)
    local current = getPosition()
    if current == nil then
        position.heading = "u"
    else
        position.heading = current.heading
    end
    setPosition(position)
end

local function updateHeading(heading)
    local position = getPosition()
    if position == nil then
        position.x = 0
        position.y = 0
        position.z = 0
    end

    position.heading = heading
    setPosition(position)
end

local function collectInventory()
    local items = {}
    for i = 1, 16 do
        -- local item = turtle.getItemDetail(i)
        -- if item == nil then
        -- 	item = "None"
        -- end
        -- items[i] = item
        items[i] = turtle.getItemDetail(i)
    end

    local inventory = {
        items = items,
        selected_slot = turtle.getSelectedSlot(),
    }

    return inventory
end

local function selectSlot(slot)
    turtle.select(slot)

    return turtle.getItemDetail()
end

local function refuel()
    -- local success, reason = turtle.refuel()
    -- if not success then
    -- 	return reason
    -- end

    assert(turtle.refuel())

    return {
        level = turtle.getFuelLevel(),
        max = turtle.getFuelLimit(),
    }
end

local function turnLeft()
    local coords = getPosition()
    if coords == nil then
        return false, "unknown position"
    end

    if coords.heading == "n" then
        coords.heading = "w"
    elseif coords.heading == "s" then
        coords.heading = "e"
    elseif coords.heading == "e" then
        coords.heading = "n"
    elseif coords.heading == "w" then
        coords.heading = "s"
    end

    local success, reason = turtle.turnLeft()
    if success then
        setPosition(coords)
    end

    return success, reason
end

local function turnRight()
    local coords = getPosition()
    if coords == nil then
        return false, "unknown position"
    end

    if coords.heading == "n" then
        coords.heading = "e"
    elseif coords.heading == "s" then
        coords.heading = "w"
    elseif coords.heading == "e" then
        coords.heading = "s"
    elseif coords.heading == "w" then
        coords.heading = "n"
    end

    local success, reason = turtle.turnRight()
    if success then
        setPosition(coords)
    end

    return success, reason
end

local function forward()
    local coords = getPosition()
    if coords == nil then
        return false, "unknown position"
    end

    if coords.heading == "n" then
        coords.z = coords.z - 1
    elseif coords.heading == "s" then
        coords.z = coords.z + 1
    elseif coords.heading == "e" then
        coords.x = coords.x + 1
    elseif coords.heading == "w" then
        coords.x = coords.x - 1
    else
        return false, "unknown heading"
    end

    local success, reason = turtle.forward()
    if success then
        setPosition(coords)
    end

    return success, reason
end

local function back()
    local coords = getPosition()
    if coords == nil then
        return false, "unknown position"
    end

    if coords.heading == "n" then
        coords.z = coords.z + 1
    elseif coords.heading == "s" then
        coords.z = coords.z - 1
    elseif coords.heading == "e" then
        coords.x = coords.x - 1
    elseif coords.heading == "w" then
        coords.x = coords.x + 1
    else
        return false, "unknown heading"
    end

    local success, reason = turtle.back()
    if success then
        setPosition(coords)
    end

    return success, reason
end

local function up()
    local coords = getPosition()
    if coords == nil then
        return false, "unknown position"
    end

    coords.y = coords.y + 1

    local success, reason = turtle.up()
    if success then
        setPosition(coords)
    end

    return success, reason
end

local function down()
    local coords = getPosition()
    if coords == nil then
        return false, "unknown position"
    end

    coords.y = coords.y - 1

    local success, reason = turtle.down()
    if success then
        setPosition(coords)
    end

    return success, reason
end

function connect(url)
    local ws = http.websocket(url)
    if not ws then
        return false
    end

    local id = os.getComputerID()

    local status, result = pcall(ws.send, math.floor(id))
    if not status then
        print("Error sending id: ", result)
        return false
    end

    local status, result = pcall(ws.receive)
    if not status then
        print("Error getting name: ", result)
        return false
    end

    Name = result
    os.setComputerLabel(result)
    return ws
end

function collectReport()
    local position = { x = 0, y = 0, z = 0 }
    local heading = "n"
    local coords = getPosition()
    if coords ~= nil then
        position.x = coords.x
        position.y = coords.y
        position.z = coords.z
        heading = coords.heading
    end
    local fuel = {
        level = turtle.getFuelLevel(),
        max = turtle.getFuelLimit(),
    }

    local inventory = collectInventory()

    return {
        position = position,
        heading = heading,
        fuel = fuel,
        inventory = inventory,
    }
end

--#region Events

function report(ws)
    local report = collectReport()
    report.info_type = "report"

    local info = {
        type = "info",
        info = report,
    }

    ws.send(textutils.serializeJSON(info))
end

function inspect()
    local front_exists, front = turtle.inspect()
    if not front_exists then
        front = { name = "minecraft:air" }
    end
    local above_exists, above = turtle.inspectUp()
    if not above_exists then
        above = { name = "minecraft:air" }
    end
    local below_exists, below = turtle.inspectDown()
    if not below_exists then
        below = { name = "minecraft:air" }
    end

    return {
        turtle_position = getPosition(),
        above = above,
        below = below,
        front = front,
    }
end

--#endregion

-- function splitMessage(message)
--   print("Splitting message", message)
--   local commands = {}
--   for c in string.gmatch(message, "[^\r\n]+") do
--     table.insert(commands, c)
--   end
--
--   return commands
-- end

function move(ws, direction)
    if direction == "f" then
        forward()
    elseif direction == "b" then
        back()
    elseif direction == "l" then
        turnLeft()
    elseif direction == "r" then
        turnRight()
    elseif direction == "u" then
        up()
    elseif direction == "d" then
        down()
    end
end

function interpretRequest(ws, id, request)
    local response = nil
    if request.type == "inspect" then
        local block = inspect()
        response = {
            type = "inspection",
            block = block,
        }
    elseif request.type == "ping" then
        response = {
            type = "pong",
        }
    else
        print("Error unknown request:", request.type)
    end

    if response ~= nil then
        local event = {
            type = "response",
            response = {
                id = id,
                response = response,
            },
        }
        print("Sending response")
        ws.send(textutils.serializeJSON(event))
    end
end

function interpretInfalible(ws, command)
    if command.type == "ping" then
        local response = {
            type = "pong",
            id = command.id,
        }
        return response
    elseif command.type == "update_position" then
        print("Updating position")
        local new = command.coords
        new.heading = command.heading
        setPosition(new)
    elseif command.type == "inspect" then
        print("Inspecting")
        return inspect()
    elseif command.type == "get_inventory" then
        print("Sending inventory")
        return collectInventory()
    elseif command.type == "select_slot" then
        print("Selecting slot")
        return selectSlot(command.slot)
    elseif command.type == "get_report" then
        print("Sending report")
        return getPosition()
        -- local report = collectReport()
        -- report.type = "report"
        -- return report
    end
end

function interpretCommand(ws, command, messageId)
    if command.type == "request" then
        interpretRequest(ws, command.id, command.request)
    elseif command.type == "reboot" then
        print("Rebooting")
        ws.send(command.id)
        os.reboot()
    elseif command.type == "move" then
        move(ws, command.direction)
    elseif command.type == "forward" then
        print("Moving forward")
        assert(forward())
        return getPosition()
    elseif command.type == "backward" then
        print("Moving backward")
        assert(back())
        return getPosition()
    elseif command.type == "turn_left" then
        print("Turning left")
        assert(turnLeft())
        return getPosition()
    elseif command.type == "turn_right" then
        print("Turning right")
        assert(turnRight())
        return getPosition()
    elseif command.type == "up" then
        print("Moving up")
        assert(up())
        return getPosition()
    elseif command.type == "down" then
        print("Moving down")
        assert(down())
        return getPosition()
    elseif command.type == "refuel" then
        print("Refueling")
        return refuel()
    elseif command.type == "dig" then
        print("Digging")
        success, reason = turtle.dig(command.side)
        if not success then
            print("Problem digging: " .. reason)
            error(reason)
        end
        return success
    elseif command.type == "dig_up" then
        print("Digging")
        success, reason = turtle.digUp(command.side)
        if not success then
            print("Problem digging: " .. reason)
            error(reason)
        end
        return success
    elseif command.type == "dig_down" then
        print("Digging")
        success, reason = turtle.digDown(command.side)
        if not success then
            print("Problem digging: " .. reason)
            error(reason)
        end
        return success
    else
        print("Unknown command")
    end
end

-- Splits of the main part of an error message from the trace
function extractException(msg)
    local rmsg = string.reverse(msg)
    local i, _ = string.find(rmsg, " :")
    if i == nil then
        return msg
    end

    local rsplit = string.sub(rmsg, 0, i - 1)
    local split = string.reverse(rsplit)

    return split
end

function dump(o)
    if type(o) == 'table' then
        local s = '{ '
        for k, v in pairs(o) do
            if type(k) ~= 'number' then k = '"' .. k .. '"' end
            s = s .. '[' .. k .. '] = ' .. dump(v) .. ','
        end
        return s .. '} '
    else
        return tostring(o)
    end
end

function handleMessage(ws, message)
    print("Got message: ", message)
    command, reason = textutils.unserializeJSON(message)
    if command == nil then
        print(reason)
        return
    end

    if command.id == nil or command.message == nil then
        print("Got invalid command: ", command)
    end

    ws.send(textutils.serializeJSON({ type = "ok", id = command.id }))

    print("Got command type: ", command.message.type)
    local response = interpretInfalible(ws, command.message)
    if not response then
        local status, r = pcall(interpretCommand, ws, command.message, command.id)
        print("Last command status: ", status)
        if not status then
            r = extractException(r)
        end
        response = { success = status, message = r }
    end
    print("Last command response: ", dump(response))
    ws.send(textutils.serializeJSON({
        type = "response",
        id = command.id,
        response = response,
    }))
    -- if status then
    -- 	ws.send(textutils.serializeJSON({ type = "response", id = command.id, response = response }))
    -- else
    -- 	ws.send(textutils.serializeJSON({
    -- 		type = "response",
    -- 		id = command.id,
    -- 		response = { success = false, message = extractException(response) },
    -- 	}))
    -- end
end

function receive(ws)
    while true do
        report(ws)

        ws.send(textutils.serializeJSON(READY))
        local message = ws.receive()
        handleMessage(ws, message)

        -- for _, command in ipairs(command) do
        --   print(command)
        --   interpretCommand(ws, command)
        -- end
    end
end

-- Entry --

while true do
    print("Attempting to connect")
    local ws = connect("ws://{host}/ws")
    if ws then
        print("Connected")
        print("Name: ", Name)
        local position = getPosition()
        if position == nil then
            response = http.get(string.format("http://{host}/turtle/%s/position", Name))
            position, reason = textutils.unserializeJSON(response.readAll())
            setPosition(position)
        end

        print("Postion:", textutils.serialize(position))

        local status, result = pcall(receive, ws)
        if not status then
            print("Error interpreting commands", result)
        end

        ws.close()
    end

    print("Failed to connect trying again in 5 seconds")
    sleep(5)
end
