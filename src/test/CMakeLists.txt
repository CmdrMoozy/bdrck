set(bdrck-test_SOURCES

	bdrck-test.cpp

	algorithm/StringTest.cpp

	fs/IteratorTest.cpp
	fs/TemporaryStorageTest.cpp
	fs/UtilTest.cpp

	git/RepositoryTest.cpp

	json/parseTest.cpp

	params/CommandTest.cpp
	params/OptionTest.cpp

	params/detail/ParseArgumentsTest.cpp
	params/detail/ParseCommandTest.cpp
	params/detail/ParseOptionsTest.cpp

	util/ScopeExitTest.cpp

)

add_executable(bdrck-test ${bdrck-test_SOURCES})
set_target_properties(bdrck-test PROPERTIES RUNTIME_OUTPUT_DIRECTORY ${TESTS_OUTPUT_DIR})
target_link_libraries(bdrck-test

	bdrck-algorithm
	bdrck-fs
	bdrck-git
	bdrck-json
	bdrck-params
	bdrck-process
	bdrck-util
	${bdrck_LIBRARIES}

)
add_test(NAME bdrck-test COMMAND ${TESTS_OUTPUT_DIR}/bdrck-test)
