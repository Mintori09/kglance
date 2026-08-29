-- ~/.config/yazi/plugins/kglance.yazi/main.lua
-- Loaded by: require("kglance"):setup()
local _last_target = nil
local _debounce_seq = 0

return {
	setup = function()
		ps.sub("hover", function()
			local hovered = cx.active.current.hovered
			local target = hovered and tostring(hovered.url) or tostring(cx.active.current.cwd)

			-- Increment seq to invalidate any pending hover callbacks
			_debounce_seq = _debounce_seq + 1
			local seq = _debounce_seq

			ya.async(function()
				-- Wait 150ms; if a newer hover arrived, seq will differ — skip
				ya.sleep(0.10)
				if seq ~= _debounce_seq then
					return
				end
				if target == _last_target then
					return
				end
				_last_target = target

				-- Pass DBUS_SESSION_BUS_ADDRESS explicitly so the child process
				-- can connect to the user's DBus session
				local dbus = os.getenv("DBUS_SESSION_BUS_ADDRESS")
				Command("/usr/bin/kglance")
					:arg("update")
					:arg(target)
					:env("DBUS_SESSION_BUS_ADDRESS", dbus or "unix:path=/run/user/1000/bus")
					:spawn()
			end)()
		end)
	end,
}
