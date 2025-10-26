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

--#region Events

function report(ws)
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

    local report = {
        info_type = "report",
        position = position,
        heading = heading,
        fuel = fuel,
    }

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

function interpretCommand(ws, command, messageId)
    print("Got command type: ", command.type)
    if command.type == "request" then
        interpretRequest(ws, command.id, command.request)
    elseif command.type == "ping" then
        response = {
            type = "pong",
            id = command.id,
        }
        return response
    elseif command.type == "reboot" then
        print("Rebooting")
        ws.send(command.id)
        os.reboot()
    elseif command.type == "move" then
        move(ws, command.direction)
    elseif command.type == "forward" then
        print("Moving forward")
        local success, reason = forward()
        if not success then
            print("Failed to move forward: " .. reason)
        end
        return getPosition()
    elseif command.type == "backward" then
        print("Moving backward")
        local success, reason = back()
        if not success then
            print("Failed to move backward: " .. reason)
        end
        return getPosition()
    elseif command.type == "turn_left" then
        print("Turning left")
        local success, reason = turnLeft()
        if not success then
            print("Failed to turn left: " .. reason)
        end
        return getPosition()
    elseif command.type == "turn_right" then
        print("Turning right")
        local success, reason = turnRight()
        if not success then
            print("Failed to turn right: " .. reason)
        end
        return getPosition()
    elseif command.type == "up" then
        print("Moving up")
        local success, reason = up()
        if not success then
            print("Failed to move up: " .. reason)
        end
        return getPosition()
    elseif command.type == "down" then
        print("Moving down")
        local success, reason = down()
        if not success then
            print("Failed to move down: " .. reason)
        end
        return getPosition()
    elseif command.type == "update_position" then
        print("Updating position")
        local new = command.coords
        new.heading = command.heading
        setPosition(new)
    elseif command.type == "inspect" then
        print("Inspecting")
        return inspect()
    elseif command.type == "dig" then
        print("Digging")
        success, reason = turtle.dig(command.side)
        if not success then
            print("Problem digging: " .. reason)
        end
        return success
    elseif command.type == "dig_up" then
        print("Digging")
        success, reason = turtle.digUp(command.side)
        if not success then
            print("Problem digging: " .. reason)
        end
        return success
    elseif command.type == "dig_down" then
        print("Digging")
        success, reason = turtle.digDown(command.side)
        if not success then
            print("Problem digging: " .. reason)
        end
        return success
    else
        print("Unknown command")
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

    local response = interpretCommand(ws, command.message, command.id)
    ws.send(textutils.serializeJSON({ type = "response", id = command.id, response = response }))
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
