"""
Example Facts Collector Plugin for Driftless (Python)

This plugin demonstrates how to create custom facts collectors using Python
and Pyodide for WebAssembly compilation.

Note: This is a conceptual example. Full Pyodide integration would require
additional build tooling and WASM compilation steps.
"""

import json
from typing import Dict, Any


def get_facts_collectors() -> str:
    """Return JSON array of facts collector definitions."""
    collectors = [
        {
            "name": "python_system_info",
            "config_schema": {
                "type": "object",
                "properties": {
                    "include_cpu": {
                        "type": "boolean",
                        "default": True,
                        "description": "Include CPU information"
                    },
                    "include_memory": {
                        "type": "boolean",
                        "default": True,
                        "description": "Include memory information"
                    }
                }
            }
        },
        {
            "name": "python_network_interfaces",
            "config_schema": {
                "type": "object",
                "properties": {
                    "include_loopback": {
                        "type": "boolean",
                        "default": False,
                        "description": "Include loopback interfaces"
                    }
                }
            }
        }
    ]

    return json.dumps(collectors)


def execute_facts_collector(name: str, config_json: str) -> str:
    """Execute a facts collector."""
    try:
        config = json.loads(config_json)

        if name == "python_system_info":
            return execute_system_info_collector(config)
        elif name == "python_network_interfaces":
            return execute_network_interfaces_collector(config)
        else:
            return json.dumps({
                "error": f"Unknown collector: {name}"
            })
    except Exception as e:
        return json.dumps({
            "error": f"Collector execution failed: {str(e)}"
        })


def execute_system_info_collector(config: Dict[str, Any]) -> str:
    """Collect basic system information."""
    facts = {}

    if config.get("include_cpu", True):
        # In a real implementation, this would use platform-specific APIs
        facts["cpu"] = {
            "architecture": "simulated",  # platform.machine()
            "cores": 4,  # os.cpu_count()
            "python_version": "3.11.0"  # platform.python_version()
        }

    if config.get("include_memory", True):
        # In a real implementation, this would use psutil or similar
        facts["memory"] = {
            "total_gb": 16,  # simulated
            "available_gb": 8,  # simulated
            "used_percent": 50.0  # simulated
        }

    facts["timestamp"] = "2024-01-25T10:30:00Z"  # simulated

    return json.dumps(facts)


def execute_network_interfaces_collector(config: Dict[str, Any]) -> str:
    """Collect network interface information."""
    include_loopback = config.get("include_loopback", False)

    # In a real implementation, this would use socket or psutil
    interfaces = {
        "eth0": {
            "addresses": ["192.168.1.100"],
            "mac": "00:11:22:33:44:55",
            "status": "up"
        },
        "wlan0": {
            "addresses": ["192.168.1.101"],
            "mac": "66:77:88:99:AA:BB",
            "status": "up"
        }
    }

    if not include_loopback:
        interfaces.pop("lo", None)
    else:
        interfaces["lo"] = {
            "addresses": ["127.0.0.1"],
            "mac": None,
            "status": "up"
        }

    return json.dumps(interfaces)


# Other required plugin functions (return empty arrays)
def get_task_definitions() -> str:
    """Return empty array - no tasks in this plugin."""
    return "[]"


def get_template_extensions() -> str:
    """Return empty array - no template extensions in this plugin."""
    return "[]"


def get_log_sources() -> str:
    """Return empty array - no log sources in this plugin."""
    return "[]"


def get_log_parsers() -> str:
    """Return empty array - no log parsers in this plugin."""
    return "[]"


def get_log_filters() -> str:
    """Return empty array - no log filters in this plugin."""
    return "[]"


def get_log_outputs() -> str:
    """Return empty array - no log outputs in this plugin."""
    return "[]"


# WebAssembly/JavaScript interop (for Pyodide)
def _setup_js_bindings():
    """Set up JavaScript bindings for WASM environment."""
    # This would be handled by Pyodide's JavaScript interop
    pass


if __name__ == "__main__":
    # Test the plugin functions
    print("Facts collectors:", get_facts_collectors())

    # Test system info collector
    config = json.dumps({"include_cpu": True, "include_memory": True})
    result = execute_facts_collector("python_system_info", config)
    print("System info:", result)

    # Test network interfaces collector
    config = json.dumps({"include_loopback": False})
    result = execute_facts_collector("python_network_interfaces", config)
    print("Network interfaces:", result)