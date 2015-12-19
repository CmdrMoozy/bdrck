set(bdrck-test_SOURCES

	bdrck-test.cpp

	util/ScopeExitTest.cpp

)

add_executable(bdrck-test ${bdrck-test_SOURCES})
target_link_libraries(bdrck-test bdrck-cwrap bdrck-process bdrck-util)
add_test(bdrck-test bdrck-test)
