# - Try to find MySQL.
# Once done this will define:
# MySQL_FOUND			- If false, do not try to use MySQL.
# MySQL_INCLUDE_DIRS	- Where to find mysql.h, etc.
# MySQL_LIBRARIES		- The libraries to link against.
# MySQL_VERSION_STRING	- Version in a string of MySQL.
#
# For backwards compatibility the following variables are also set:
# MYSQL_INCLUDE_DIRS	- Where to find mysql.h, etc.
# MYSQL_LIBRARIES		- The libraries to link against.
# MYSQL_VERSION_STRING	- Version in a string of MySQL.
#
# This module reads hints about search locations from variables::
#
# MYSQL_ROOT_DIR		- The root directory of the MySQL installation.
# MYSQL_INCLUDE_DIR	- The directory containing the MySQL headers.
# MYSQL_LIBRARY_DIR	- The directory containing the MySQL libraries.
#
# Variables used internally by this module:
# MySQL_FOUND			- If false, do not try to use MySQL.
# MySQL_INCLUDE_DIRS	- Where to find mysql.h, etc.
# MySQL_LIBRARIES		- The libraries to link against.
# MySQL_VERSION_STRING	- Version in a string of MySQL.

set(MySQL_FOUND FALSE)
set(MySQL_INCLUDE_DIRS "")
set(MySQL_LIBRARIES "")
set(MySQL_VERSION_STRING "")

find_path(MySQL_INCLUDE_DIR
    NAMES mysql.h
    PATHS /usr/include/mysql
          /usr/local/include/mysql
          /opt/mysql/include/mysql
          /opt/local/include/mysql
          /usr/mysql/include/mysql
)

find_library(MySQL_LIBRARY
    NAMES mysqlclient
    PATHS /usr/lib
          /usr/local/lib
          /opt/mysql/lib
          /opt/local/lib
          /usr/mysql/lib
)

if(MySQL_INCLUDE_DIR AND MySQL_LIBRARY)
    set(MySQL_FOUND TRUE)
    set(MySQL_INCLUDE_DIRS ${MySQL_INCLUDE_DIR})
    set(MySQL_LIBRARIES ${MySQL_LIBRARY})
    
    # Try to find the version
    if(MySQL_INCLUDE_DIR)
        file(READ "${MySQL_INCLUDE_DIR}/mysql_version.h" _mysql_version_header)
        
        string(REGEX MATCH "define[ \t]+MYSQL_VERSION_ID[ \t]+([0-9]+)" _mysql_version_match "${_mysql_version_header}")
        set(MySQL_VERSION_ID "${CMAKE_MATCH_1}")
        
        string(REGEX MATCH "define[ \t]+MYSQL_SERVER_VERSION[ \t]+\"([0-9\.]+)" _mysql_version_match "${_mysql_version_header}")
        set(MySQL_VERSION_STRING "${CMAKE_MATCH_1}")
    endif()
endif()

# Handle the QUIETLY and REQUIRED arguments and set MySQL_FOUND to TRUE if
# all listed variables are TRUE
include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(MySQL
    REQUIRED_VARS MySQL_LIBRARY MySQL_INCLUDE_DIR
    VERSION_VAR MySQL_VERSION_STRING
)

mark_as_advanced(MySQL_INCLUDE_DIR MySQL_LIBRARY)

# For backwards compatibility
set(MYSQL_INCLUDE_DIRS ${MySQL_INCLUDE_DIRS})
set(MYSQL_LIBRARIES ${MySQL_LIBRARIES})
set(MYSQL_VERSION_STRING ${MySQL_VERSION_STRING})