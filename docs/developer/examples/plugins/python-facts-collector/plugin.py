"""
Example Facts Collector Plugin for Driftless (Python)

This plugin demonstrates how to create custom facts collectors using Python
and Pyodide for WebAssembly compilation.
"""

import json
import platform
import socket
from typing import Dict, Any

try:
    import psutil
    HAS_PSUTIL = True
except ImportError:
    HAS_PSUTIL = False


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
    """Collect basic system information using real APIs."""
    facts = {}

    if config.get("include_cpu", True):
        if HAS_PSUTIL:
            try:
                cpu_info = {
                    "architecture": platform.machine(),
                    "cores": psutil.cpu_count(logical=True),
                    "physical_cores": psutil.cpu_count(logical=False),
                    "cpu_percent": psutil.cpu_percent(interval=1),
                    "python_version": platform.python_version()
                }
                facts["cpu"] = cpu_info
            except Exception as e:
                facts["cpu"] = {"error": f"Failed to collect CPU info: {str(e)}"}
        else:
            facts["cpu"] = {
                "architecture": platform.machine(),
                "cores": "psutil not available",
                "python_version": platform.python_version()
            }

    if config.get("include_memory", True):
        if HAS_PSUTIL:
            try:
                memory = psutil.virtual_memory()
                facts["memory"] = {
                    "total_gb": round(memory.total / (1024**3), 2),
                    "available_gb": round(memory.available / (1024**3), 2),
                    "used_gb": round(memory.used / (1024**3), 2),
                    "used_percent": memory.percent
                }
            except Exception as e:
                facts["memory"] = {"error": f"Failed to collect memory info: {str(e)}"}
        else:
            facts["memory"] = {"error": "psutil not available for memory collection"}

    # Add timestamp
    from datetime import datetime
    facts["timestamp"] = datetime.utcnow().isoformat() + "Z"

    return json.dumps(facts)


def execute_network_interfaces_collector(config: Dict[str, Any]) -> str:
    """Collect network interface information using socket APIs."""
    include_loopback = config.get("include_loopback", False)

    try:
        interfaces = {}

        # Get network interface information
        if hasattr(socket, 'getaddrinfo') and hasattr(socket, 'gethostname'):
            try:
                hostname = socket.gethostname()
                interfaces["hostname"] = hostname

                # Get IP addresses for hostname
                try:
                    addr_info = socket.getaddrinfo(hostname, None)
                    ip_addresses = list(set(
                        addr[4][0] for addr in addr_info
                        if addr[4][0] not in ('127.0.0.1', '::1') or include_loopback
                    ))
                    interfaces["hostname_ips"] = ip_addresses
                except Exception:
                    interfaces["hostname_ips"] = []

            except Exception as e:
                interfaces["hostname"] = f"error: {str(e)}"

        # Try to get more detailed interface info with psutil if available
        if HAS_PSUTIL:
            try:
                net_if_addrs = psutil.net_if_addrs()
                net_if_stats = psutil.net_if_stats()

                for interface_name, addrs in net_if_addrs.items():
                    if not include_loopback and interface_name.startswith(('lo', 'Loopback')):
                        continue

                    interface_info = {
                        "addresses": [],
                        "mac": None,
                        "status": "unknown"
                    }

                    for addr in addrs:
                        if addr.family == socket.AF_INET:
                            interface_info["addresses"].append(addr.address)
                        elif addr.family == socket.AF_INET6:
                            interface_info["addresses"].append(addr.address)
                        elif addr.family == psutil.AF_LINK:
                            interface_info["mac"] = addr.address

                    # Get interface status
                    if interface_name in net_if_stats:
                        stats = net_if_stats[interface_name]
                        interface_info["status"] = "up" if stats.isup else "down"
                        interface_info["mtu"] = stats.mtu

                    interfaces[interface_name] = interface_info

            except Exception as e:
                interfaces["psutil_error"] = str(e)

        # Fallback if psutil is not available
        if not HAS_PSUTIL or len(interfaces) <= 2:  # Only hostname info
            # Basic fallback using socket
            try:
                # Get local IP (this is a simplified approach)
                s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
                s.connect(("8.8.8.8", 80))
                local_ip = s.getsockname()[0]
                s.close()

                interfaces["primary_interface"] = {
                    "addresses": [local_ip],
                    "mac": "unknown (psutil not available)",
                    "status": "up"
                }
            except Exception:
                interfaces["network_info"] = "Limited network info available (psutil not installed)"

        return json.dumps(interfaces)

    except Exception as e:
        return json.dumps({
            "error": f"Failed to collect network interface information: {str(e)}"
        })


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