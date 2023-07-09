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

  return false;
end

function receive(ws)
  while true do
    print(ws.receive())
  end
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
  local position = {x = 0, y = 0, z = 0 }
  local heading = "North"
  local fuel = {
    level = turtle.getFuelLevel(),
    max = turtle.getFuelLimit(),
  }

  local report = {
    type = "report",
    position = position,
    heading = heading,
    fuel = fuel,
  }

  ws.send(textutils.serializeJSON(report))
end

function inspect(ws)
  local exists, block = turtle.inspect() 
  if not exists then
    block = {}
    block.type = "air"
  elseif hasValue(KNOWNBLOCKS, block.name) then
    block.type = block.name
  else
    block.type = "other"
  end

  local inspection = {
    type = "inspection",
    block = block,
  }

  print(inspection)
  ws.send(textutils.serializeJSON(inspection))
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

function interpretCommand(ws, command)
  print("Got command type: ", command.type)
  if command.type == "forward" then
    print("Moving forward")
    turtle.forward()
  elseif command.type == "back" then
    print("Moving back")
    turtle.back()
  elseif command.type == "turn_left" then
    print("Turning left")
    turtle.turnLeft()
  elseif command.type == "turn_right" then
    print("Turning right")
    turtle.turnRight()
  elseif command.type == "reboot" then
    print("Rebooting")
    os.reboot()
  elseif command.type == "inspect" then
    print("Inspecting")
    inspect(ws)
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

    if command.id == nil or command.command == nil then
      print("Got invalid command: ", command)
    end

    ws.send(textutils.serializeJSON({ type = "ok", id = command.id }))

    interpretCommand(ws, command.command)
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
  local ws = connect("ws://127.0.0.1:8080")
  if (ws) then

    print("Connected")
    print("Name: ", Name)
    local status, result = pcall(receive, ws)
    if not status then
      print("Error interpreting commands", result)
    end

    ws.close()
  end

  print("Failed to connect trying again in 5 seconds")
  sleep(5)
end

